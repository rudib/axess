use broadcaster::BroadcastChannel;
use crate::{midi::{MidiConnection, Midi}, payload::{PayloadConnection, UiPayload, ConnectToMidiPorts}, FractalResult, FractalResultVoid, utils::filter_first};
use fractal_protocol::{message::{FractalMessage, FractalMessageWrapper}, model::{model_code, FractalDevice}, message2::validate_and_decode_message, common::{disconnect_from_controller, wrap_msg, get_current_preset_name, get_firmware_version}};
use std::{time::Duration, thread};
use log::trace;
use tokio::runtime::Runtime;
use futures::executor::block_on;
use crate::FractalCoreError;

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>,
    midi: Midi,
    midi_connection: Option<MidiConnection<BroadcastChannel<FractalMessageWrapper>>>,
    device: Option<FractalDevice>,
    midi_messages: BroadcastChannel<FractalMessageWrapper>
}

impl UiBackend {    
    pub fn spawn() -> UiApi {
        let chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        let mut backend = UiBackend {
            channel: chan,
            midi: Midi::new().unwrap(),
            midi_connection: None,
            midi_messages: BroadcastChannel::new(),
            device: None
        };

        thread::Builder::new().name("Backend".into()).spawn(move || {
            trace!("Backend initialized");
            
            let mut runtime = Runtime::new().unwrap();
            runtime.block_on(backend.main_loop());

            trace!("Backend shutting down");
        }).unwrap();

        api
    }

    async fn main_loop(&mut self) {
        loop {
            let msg = self.channel.recv().await;
            if let Some(ref msg) = msg {
                trace!("Backend message received: {:?}", msg);
            }

            match msg {  

                Some(UiPayload::Connection(c)) => {
                    self.connection(c).await;
                },
                Some(_) => {

                },
                None => {
                    println!("end of stream!");
                    break;
                }
            }
        }
    }

    async fn send(&self, msg: UiPayload) -> FractalResultVoid {        
        self.channel.send(&msg).await?;
        trace!("Backend sent message: {:?}", msg);
        Ok(())
    }

    async fn connection(&mut self, msg: PayloadConnection) -> FractalResultVoid {
        match msg {
            PayloadConnection::ListMidiPorts => {
                let midi_ports = self.midi.detect_midi_ports()?;
                self.send(UiPayload::Connection(PayloadConnection::DetectedMidiPorts {
                    ports: midi_ports
                })).await?;
            },

            PayloadConnection::ConnectToMidiPorts(ref ports) => {
                match self.connect(ports).await {
                    Ok(device) => {
                        self.device = Some(device.clone());
                        self.send(UiPayload::Connection(PayloadConnection::Connected { device: device })).await?;
                        self.on_connect();
                    },
                    Err(e) => {
                        self.send(UiPayload::Connection(PayloadConnection::ConnectionFailed(e))).await?
                    }
                }
            },

            PayloadConnection::TryToAutoConnect => {
                let midi_ports = self.midi.detect_midi_ports()?;
                let fractal_devices = midi_ports.detect_fractal_devices();
                trace!("Detected Fractal Devices: {:?}", &fractal_devices);

                if fractal_devices.len() == 1 {
                    trace!("Found a single device. Will try to connect.");
                    let fractal_device = fractal_devices.first().unwrap();
                    self.send(UiPayload::Connection(PayloadConnection::ConnectToMidiPorts(ConnectToMidiPorts {
                        input_port: fractal_device.input_port_name.clone(),
                        output_port: fractal_device.output_port_name.clone()                  
                    }))).await?;
                } else {
                    self.send(UiPayload::Connection(PayloadConnection::AutoConnectDeviceNotFound)).await?;
                }
            },
            
            PayloadConnection::Disconnect => {
                match (self.midi_connection.take(), self.device.take()) {
                    (Some(mut midi_connection), Some(device)) => {
                        midi_connection.output.send(&disconnect_from_controller(device.model))?;
                    },
                    _ => {}
                }
                
                self.send(UiPayload::Connection(PayloadConnection::Disconnected)).await?;
                self.on_disconnect();

            },

            _ => {}
        }

        Ok(())
    }

    fn on_connect(&mut self) {
        match (&mut self.midi_connection, &self.device) {
            (Some(ref mut midi), Some(ref device)) => {
                midi.output.send(&get_current_preset_name(device.model));
            },
            _ => {}
        }
        
        trace!("start the poller!");
    }

    fn on_disconnect(&mut self) {
        trace!("stop the poller!");
    }

    fn midi_message_callback(msg: &[u8], ctx: &mut BroadcastChannel<FractalMessageWrapper>) {
        if let Some(msg) = validate_and_decode_message(msg) {
            trace!("Raw MIDI message: {:?}", msg);
            // todo: can we make this async?
            block_on(ctx.send(&msg)).unwrap();
        }
    }

    async fn connect(&mut self, ports: &ConnectToMidiPorts) -> FractalResult<FractalDevice> {
        let timeout = Duration::from_millis(100);

        let mut connection = self.midi.connect_to(&ports.input_port, &ports.output_port,
            UiBackend::midi_message_callback, self.midi_messages.clone())?;

        // send a message that should reply to us with the model
        connection.output.send(&wrap_msg(vec![0x7F, 0x00]))?;

        // retrieve the model
        
        let model = filter_first(&mut self.midi_messages, |msg| {
            match msg.message {
                FractalMessage::MultipurposeResponse { function_id, response_code} 
                    if function_id == 0 && response_code == 0 => 
                {
                    msg.model
                },
                _ => None
            }
        }, timeout).await;
        let model = model.map_err (|_| FractalCoreError::MissingValue("Model".into()))?;
        trace!("Detected Fractal Model {:?}", model);

        // request the firmware version
        connection.output.send(&get_firmware_version(model_code(model)))?;

        let firmware = filter_first(&mut self.midi_messages, |msg| {
            match msg.message {
                FractalMessage::FirmwareVersion { major, minor } => {
                    Some((major, minor))
                },
                _ => None
            }
        }, timeout).await;
        let firmware = firmware.map_err(|_| FractalCoreError::MissingValue("Firmware".into()))?;
        trace!("Detected firmware {:?}", firmware);

        self.midi_connection = Some(connection);
        let device = FractalDevice {
            firmware: firmware,
            model: model
        };
        Ok(device)
    }
}