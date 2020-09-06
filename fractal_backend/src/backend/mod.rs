use broadcaster::BroadcastChannel;
use crate::{midi::{MidiConnection, Midi}, payload::{PayloadConnection, UiPayload, ConnectToMidiPorts}, FractalResult, FractalResultVoid, utils::filter_first};
use fractal_protocol::{message::{FractalMessage, FractalMessageWrapper}, model::{model_code, FractalDevice}, message2::validate_and_decode_message, common::{disconnect_from_controller, wrap_msg, get_current_preset_name, get_firmware_version, get_current_scene_name}};
use std::{time::Duration, thread, pin::Pin};
use log::{error, trace};
use tokio::runtime::Runtime;
use tokio::stream::{pending, Stream};
use futures::{Future, executor::block_on, StreamExt, future::Either};
use crate::FractalCoreError;

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>,
    midi: Midi,
    device: Option<ConnectedDevice>,
    status_poller: Pin<Box<dyn Stream<Item = tokio::time::Instant>>>
}

struct ConnectedDevice {
    midi_connection: MidiConnection<BroadcastChannel<FractalMessageWrapper>>,
    device: FractalDevice,
    midi_messages: BroadcastChannel<FractalMessageWrapper>,
    state: DeviceState
}

impl ConnectedDevice {
    async fn update_state(&mut self) -> FractalResult<bool> {
        let timeout = Duration::from_millis(100);

        self.midi_connection.output.send(&get_current_preset_name(self.device.model))?;
        self.midi_connection.output.send(&get_current_scene_name(self.device.model))?;

        let mut device_state = DeviceState::default();
        filter_first(&mut self.midi_messages, |msg| {
            match msg.message {
                FractalMessage::PresetName(preset_number, preset_name) => {
                    device_state.preset_number = preset_number;
                    device_state.preset_name = preset_name;
                }
                _ => ()
            }
            Some(())
        }, timeout).await?;

        filter_first(&mut self.midi_messages, |msg| {
            if let FractalMessage::SceneName(scene, name) = msg.message {
                device_state.scene_number = scene;
                device_state.scene_name = name;
            }
            Some(())
        }, timeout).await?;

        if device_state != self.state {
            self.state = device_state;
            return Ok(true);
        }

        Ok(false)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct DeviceState {
    preset_number: u32,
    preset_name: String,
    scene_number: u8,
    scene_name: String
}

impl UiBackend {    
    pub fn spawn() -> UiApi {
        let chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        
        thread::Builder::new().name("Backend".into()).spawn(move || {
            let mut backend = UiBackend {
                channel: chan,
                midi: Midi::new().unwrap(),
                device: None,
                status_poller: Box::pin(pending())
            };

            trace!("Backend initialized");
            
            let mut runtime = Runtime::new().unwrap();
            runtime.block_on(backend.main_loop());

            trace!("Backend shutting down");
        }).unwrap();

        api
    }

    async fn main_loop(&mut self) {
        loop {
            enum PendingAction { Message(UiPayload), EndOfMessagesChannel, Poll }
            
            let action = {
                let r = futures::future::select(self.channel.recv(), self.status_poller.next()).await;
                match r {
                    Either::Left(x) => {                    
                        if let Some(m) = x.0 {
                            PendingAction::Message(m)
                        } else {
                            PendingAction::EndOfMessagesChannel
                        }
                    },
                    Either::Right(_) => {
                        PendingAction::Poll
                    }
                }
            };

            match action {
                PendingAction::Message(msg) => {
                    trace!("Backend message received: {:?}", msg);

                    match msg {
                        UiPayload::Connection(c) => {
                            self.connection(c).await;
                        },
                        _ => {}
                    };
                },
                PendingAction::EndOfMessagesChannel => {
                    println!("end of stream!");
                    break;
                },
                PendingAction::Poll => {
                    self.status_poll().await;
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
                        let info = device.device.clone();
                        self.device = Some(device);
                        self.send(UiPayload::Connection(PayloadConnection::Connected { device: info })).await?;
                        self.send_device_state().await?;
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
                if let Some(mut device) = self.device.take() {
                    device.midi_connection.output.send(&disconnect_from_controller(device.device.model))?;
                }
                
                self.send(UiPayload::Connection(PayloadConnection::Disconnected)).await?;
                self.on_disconnect();

            },

            _ => {}
        }

        Ok(())
    }

    /// request the basic infos from the device that might have changed
    async fn status_poll(&mut self) -> FractalResultVoid {
        let mut updated = false;

        if let Some(ref mut connected_device) = self.device {
            match connected_device.update_state().await {
                Ok(true) => {
                    updated = true;
                },
                Ok(false) => {},
                Err(e) => {
                    // failed to poll the device. disconnect.
                    error!("Polling failed: {:?}", e);
                }
            }
        }

        if updated {
            self.send_device_state().await?;
        }

        Ok(())
    }

    async fn send_device_state(&self) -> FractalResultVoid {
        if let Some(ref connected_device) = self.device {
            let state = &connected_device.state;
            self.send(UiPayload::DeviceState(crate::payload::DeviceState::PresetAndScene(crate::payload::PresetAndScene {
                preset: state.preset_number as u16,
                preset_name: state.preset_name.clone(),
                scene: state.scene_number,
                scene_name: state.scene_name.clone()
            }))).await?;
        }

        Ok(())
    }

    fn on_connect(&mut self) {
        self.status_poller = Box::pin(tokio::time::interval(Duration::from_millis(100)));
        
    }

    fn on_disconnect(&mut self) {
        self.status_poller = Box::pin(pending());
    }

    fn midi_message_callback(msg: &[u8], ctx: &mut BroadcastChannel<FractalMessageWrapper>) {
        if let Some(msg) = validate_and_decode_message(msg) {
            trace!("Raw MIDI message: {:?}", msg);
            // todo: can we make this async?
            block_on(ctx.send(&msg)).unwrap();
        }
    }

    async fn connect(&mut self, ports: &ConnectToMidiPorts) -> FractalResult<ConnectedDevice> {
        let timeout = Duration::from_millis(100);
        let mut midi_messages = BroadcastChannel::new();

        let mut connection = self.midi.connect_to(&ports.input_port, &ports.output_port,
            UiBackend::midi_message_callback, midi_messages.clone())?;

        // send a message that should reply to us with the model
        connection.output.send(&wrap_msg(vec![0x7F, 0x00]))?;

        // retrieve the model
        
        let model = filter_first(&mut midi_messages, |msg| {
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

        let firmware = filter_first(&mut midi_messages, |msg| {
            match msg.message {
                FractalMessage::FirmwareVersion { major, minor } => {
                    Some((major, minor))
                },
                _ => None
            }
        }, timeout).await;
        let firmware = firmware.map_err(|_| FractalCoreError::MissingValue("Firmware".into()))?;
        trace!("Detected firmware {:?}", firmware);

        
        let device = FractalDevice {
            firmware: firmware,
            model: model
        };

        let mut connected_device = ConnectedDevice {
            device: device,
            midi_connection: connection,
            midi_messages: midi_messages,
            state: DeviceState::default()
        };
        
        connected_device.update_state().await?;

        Ok(connected_device)
    }
}