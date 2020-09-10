use broadcaster::BroadcastChannel;
use crate::{payload::{PayloadConnection, UiPayload}, FractalResult, FractalResultVoid, utils::filter_first};
use crate::transport::{Transport, midi::{MidiConnection, Midi}, TransportConnection, serial::TransportSerial, Endpoint};
use fractal_protocol::{message::{FractalMessage, FractalMessageWrapper}, model::{model_code, FractalDevice}, message2::validate_and_decode_message, common::{disconnect_from_controller, wrap_msg, get_current_preset_name, get_firmware_version, get_current_scene_name, set_preset_number, set_scene_number}, functions::FractalFunction, message2::SYSEX_START, message2::SYSEX_MANUFACTURER_BYTE1, message2::SYSEX_MANUFACTURER_BYTE2, message2::SYSEX_MANUFACTURER_BYTE3, message2::SYSEX_END, buffer::MessagesBuffer};
use std::{time::Duration, thread, pin::Pin};
use log::{error, trace};
use tokio::runtime::Runtime;
use tokio::stream::{pending, Stream};
use futures::{executor::block_on, StreamExt, future::{self, Either}};
use crate::FractalCoreError;

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>,
    transports: Vec<Box<dyn Transport>>,
    device: Option<ConnectedDevice>,
    status_poller: Pin<Box<dyn Stream<Item = tokio::time::Instant>>>
}

struct ConnectedDevice {
    transport_endpoint: Box<dyn TransportConnection>,
    device: FractalDevice,
    midi_messages: BroadcastChannel<FractalMessageWrapper>,
    state: DeviceState,
    //midi_channel: u8
}

impl ConnectedDevice {
    async fn update_state(&mut self) -> FractalResult<bool> {
        let timeout = Duration::from_millis(500);

        let channel = self.midi_messages.clone();

        self.transport_endpoint.write(&get_current_preset_name(self.device.model))?;
        self.transport_endpoint.write(&get_current_scene_name(self.device.model))?;

        let mut device_state = DeviceState::default();

        let scan = channel
            .scan(0 as u32, |state, msg| {
                match &msg.message {
                    FractalMessage::PresetName(preset_number, preset_name) => {
                        device_state.preset_number = *preset_number;
                        device_state.preset_name = preset_name.clone();
                        *state += 1;
                    },
                    FractalMessage::SceneName(scene, name) => {
                        device_state.scene_number = *scene;
                        device_state.scene_name = name.clone();
                        *state += 1;
                    },
                    _ => ()
                }

                future::ready(if *state < 2 { Some(msg) } else { None })
            });
        
        let t = tokio::time::timeout(timeout, scan.collect::<Vec<_>>()).await;
        if t.is_err() {
            return Err(FractalCoreError::Timeout);
        }

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
    pub fn spawn() -> FractalResult<UiApi> {
        let chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        let midi = Midi::new()?;
        let serial = TransportSerial::new();
        
        thread::Builder::new().name("Backend".into()).spawn(move || {
            let mut backend = UiBackend {
                channel: chan,                
                device: None,
                status_poller: Box::pin(pending()),
                transports: vec![
                    Box::new(midi),
                    Box::new(serial)
                ]
            };

            trace!("Backend initialized");
            
            let mut runtime = Runtime::new().unwrap();
            runtime.block_on(backend.main_loop());

            trace!("Backend shutting down");
        }).unwrap();

        Ok(api)
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
                        UiPayload::DeviceState(crate::payload::DeviceState::SetPreset{ preset }) => {
                            if let Some(ref mut device) = self.device {
                                let bank = (preset / 128) as u8;
                                let patch = (preset % 128) as u8;
                                device.transport_endpoint.write(&vec![0xB0, 0x00, bank]);
                                device.transport_endpoint.write(&vec![0xC0, patch]);
                            }

                            tokio::time::delay_for(Duration::from_millis(10)).await;
                            self.status_poll().await;
                        },
                        UiPayload::DeviceState(crate::payload::DeviceState::SetScene { scene }) => {
                            if let Some(ref mut device) = self.device {
                                device.transport_endpoint.write(&set_scene_number(device.device.model, scene));
                            }

                            tokio::time::delay_for(Duration::from_millis(10)).await;
                            self.status_poll().await;
                        }
                        _ => {}
                    };
                },
                PendingAction::EndOfMessagesChannel => {
                    trace!("end of stream!");
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
            PayloadConnection::ListEndpoints => {
                let mut detected_endpoints = vec![];

                for transport in &self.transports {
                    if let Ok(endpoints) = transport.detect_endpoints() {
                        for endpoint in endpoints {
                            detected_endpoints.push(Endpoint {
                                transport_id: transport.id().clone(),
                                transport_endpoint: endpoint
                            });
                        }
                    }
                }

                self.send(UiPayload::Connection(PayloadConnection::DetectedEndpoints {
                    endpoints: detected_endpoints
                })).await?;
            },

            PayloadConnection::ConnectToEndpoint(endpoint) => {
                match self.connect(&endpoint).await {
                    Ok(device) => {
                        let info = device.device.clone();
                        self.device = Some(device);
                        self.send(UiPayload::Connection(PayloadConnection::Connected { device: info })).await?;
                        self.send_device_state().await?;
                        self.on_connect();
                    },
                    Err(e) => {
                        trace!("Connect failed: {:?}", e);
                        self.send(UiPayload::Connection(PayloadConnection::ConnectionFailed(e))).await?
                    }
                }
            },

            PayloadConnection::Disconnect => {
                if let Some(mut device) = self.device.take() {
                    device.transport_endpoint.write(&disconnect_from_controller(device.device.model))?;
                }
                
                self.send(UiPayload::Connection(PayloadConnection::Disconnected)).await?;
                self.on_disconnect();                
            }

            /*
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
            */

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
                    self.channel.send(&UiPayload::Connection(PayloadConnection::Disconnect)).await?;
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
        self.status_poller = Box::pin(tokio::time::interval(Duration::from_millis(1000)));
        
    }

    fn on_disconnect(&mut self) {
        self.status_poller = Box::pin(pending());
    }

    async fn connect(&mut self, endpoint: &Endpoint) -> FractalResult<ConnectedDevice> {
        let timeout = Duration::from_millis(200);
        
        let transport = self.transports.iter().find(|t| t.id() == endpoint.transport_id).ok_or(FractalCoreError::Other("Transport not found".into()))?;
        let mut connection = transport.connect(&endpoint.transport_endpoint)?;

        let mut midi_messages = BroadcastChannel::<FractalMessageWrapper>::new();

        {
            let receiver = connection.get_receiver().clone();
            let midi_messages = midi_messages.clone();

            thread::spawn(move || {
                let mut messages_buffer = MessagesBuffer::new();
                loop {
                    if let Ok(msg) = receiver.recv() {
                        if let Some(msg) = messages_buffer.parse(&msg) {
                            trace!("Received SYSEX message: {:?}", msg);
                            block_on(midi_messages.send(&msg)).unwrap();
                        }
                    } else {
                        break;
                    }
                }

                trace!("stop bridge");
            });
        }

        // send a message that should reply to us with the model
        connection.write(&wrap_msg(vec![0x7F, 0x00]))?;
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
        connection.write(&get_firmware_version(model_code(model)))?;

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

        /*
        // get the midi channel
        connection.write(&wrap_msg(vec![model_code(model), FractalFunction::GET_MIDI_CHANNEL as u8]));

        let midi_channel = filter_first(&mut midi_messages, |msg| {
            match msg.message {
                FractalMessage::MultipurposeResponse { function_id, response_code } if function_id == FractalFunction::GET_MIDI_CHANNEL as u8 => {
                    Some(response_code)
                },
                _ => { None }
            }
        }, timeout).await;
        let midi_channel = midi_channel.map_err(|_| FractalCoreError::MissingValue("MIDI channel".into()))?;
        trace!("MIDI channel: {}", midi_channel);
        */

        let device = FractalDevice {
            firmware: firmware,
            model: model
        };

        let mut connected_device = ConnectedDevice {
            device: device,
            //midi_channel: midi_channel,
            transport_endpoint: connection,
            midi_messages: midi_messages,
            state: DeviceState::default()
        };
        
        connected_device.update_state().await?;

        Ok(connected_device)
    }
}