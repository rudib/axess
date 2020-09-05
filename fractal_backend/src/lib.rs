extern crate broadcaster;
extern crate fractal_protocol;

use std::{thread, time::Duration};
use fractal_core::{FractalCoreError, midi::{Midi, MidiPorts, MidiConnection}};
use broadcaster::BroadcastChannel;
use futures::executor::block_on;
use log::{error, trace};
use fractal_protocol::{message2::validate_and_decode_message, common::{get_firmware_version, wrap_msg}, message::{FractalMessage, FractalMessageWrapper}, model::{FractalDevice, model_code}};
use utils::{channel_map_and_filter_first_async, block_on_with_timeout};

mod utils;

#[derive(Debug, Clone)]
pub enum UiPayload {
    Connection(PayloadConnection),
    
    /// Internal
    Ping,
    
    /// Hard shutdown
    Drop
}

#[derive(Debug, Clone)]
pub enum PayloadConnection {
    ListMidiPorts,
    DetectedMidiPorts {
        ports: MidiPorts
    },
    ConnectToMidiPorts(ConnectToMidiPorts),

    TryToAutoConnect,
    AutoConnectDeviceNotFound,

    Disconnect,
    
    // Events
    ConnectionFailed(FractalCoreError),
    Connected {
        device: fractal_protocol::model::FractalDevice
    },
    Disconnected    
}

#[derive(Debug, Clone)]
pub struct ConnectToMidiPorts {
    pub input_port: String,
    pub output_port: String
}


#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>,
    midi: Midi,
    midi_connection: Option<MidiConnection<BroadcastChannel<FractalMessageWrapper>>>,
    midi_messages: BroadcastChannel<FractalMessageWrapper>
}

impl UiBackend {    
    pub fn spawn() -> UiApi {
        let mut chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        let mut backend = UiBackend {
            channel: chan,
            midi: Midi::new().unwrap(),
            midi_connection: None,
            midi_messages: BroadcastChannel::new()
        };

        thread::Builder::new().name("Backend".into()).spawn(move || {
            trace!("Backend initialized");
            loop {
                let msg = block_on(backend.channel.recv());
                if let Some(ref msg) = msg {
                    trace!("Backend message received: {:?}", msg);
                }

                match msg {  

                    Some(UiPayload::Connection(c)) => {
                        backend.connection(c);
                    },
                    Some(_) => {

                    },
                    None => {
                        println!("end of stream!");
                        break;
                    }
                }
            }

            trace!("Backend shutting down");
        }).unwrap();

        api
    }

    fn send(&self, msg: UiPayload) {        
        block_on(self.channel.send(&msg)).unwrap();
        trace!("Backend sent message: {:?}", msg);
    }

    fn connection(&mut self, msg: PayloadConnection) {
        match msg {
            PayloadConnection::ListMidiPorts => {
                let midi_ports = self.midi.detect_midi_ports().unwrap();
                self.send(UiPayload::Connection(PayloadConnection::DetectedMidiPorts {
                    ports: midi_ports
                }));
            }
            //PayloadConnection::DetectedMidiPorts { ports } => {}
            PayloadConnection::ConnectToMidiPorts(ref ports) => {
                match self.connect(ports) {
                    Ok(device) => {
                        self.send(UiPayload::Connection(PayloadConnection::Connected { device: device }));
                    },
                    Err(e) => {
                        self.send(UiPayload::Connection(PayloadConnection::ConnectionFailed(e)))
                    }
                }
            }
            PayloadConnection::TryToAutoConnect => {
                let midi_ports = self.midi.detect_midi_ports().unwrap();
                let fractal_devices = midi_ports.detect_fractal_devices();
                trace!("Detected Fractal Devices: {:?}", &fractal_devices);

                if fractal_devices.len() == 1 {
                    trace!("Found a single device. Will try to connect.");
                    let fractal_device = fractal_devices.first().unwrap();
                    self.send(UiPayload::Connection(PayloadConnection::ConnectToMidiPorts(ConnectToMidiPorts {
                        input_port: fractal_device.input_port_name.clone(),
                        output_port: fractal_device.output_port_name.clone()                  
                    })));
                } else {
                    self.send(UiPayload::Connection(PayloadConnection::AutoConnectDeviceNotFound));
                }
            }
            //PayloadConnection::AutoConnectResult(_) => {}
            //PayloadConnection::Connected => {}
            //PayloadConnection::Disconnected => {}

            PayloadConnection::Disconnect => {
                // todo: maybe send the message that we're going away? clear the broadcast?
                self.midi_connection = None;
                self.send(UiPayload::Connection(PayloadConnection::Disconnected));

            },

            _ => {}
        }
    }

    fn midi_message_callback(msg: &[u8], ctx: &mut BroadcastChannel<FractalMessageWrapper>) {
        if let Some(msg) = validate_and_decode_message(msg) {
            trace!("Fractal message: {:?}", msg);
            block_on(ctx.send(&msg)).unwrap();
        }
    }

    fn connect(&mut self, ports: &ConnectToMidiPorts) -> Result<FractalDevice, FractalCoreError> {

        let mut connection = self.midi.connect_to(&ports.input_port, &ports.output_port,
            UiBackend::midi_message_callback, self.midi_messages.clone())?;

        // send a message that should reply to us with the model
        connection.output.send(&wrap_msg(vec![0x7F, 0x00]))?;

        // retrieve the model
        let model = channel_map_and_filter_first_async(&mut self.midi_messages, |msg| {
            match msg.message {
                FractalMessage::MultipurposeResponse { function_id, response_code} 
                    if function_id == 0 && response_code == 0 => 
                {
                    msg.model
                },
                _ => None
            }
        });
        let model = block_on_with_timeout(model, Duration::from_millis(100))?;
        let model = model.ok_or(FractalCoreError::MissingValue("Model".into()))?;
        trace!("Detected Fractal Model {:?}", model);

        // request the firmware version
        connection.output.send(&get_firmware_version(model_code(model)))?;

        let firmware = channel_map_and_filter_first_async(&mut self.midi_messages, |msg| {
            match msg.message {
                FractalMessage::FirmwareVersion { major, minor } => {
                    Some((major, minor))
                },
                _ => None
            }
        });
        let firmware = block_on_with_timeout(firmware, Duration::from_millis(100))?;
        let firmware = firmware.ok_or(FractalCoreError::MissingValue("Firmware".into()))?;
        trace!("Detected firmware {:?}", firmware);

        self.midi_connection = Some(connection);
        let device = FractalDevice {
            firmware: firmware,
            model: model
        };        
        Ok(device)
    }
}