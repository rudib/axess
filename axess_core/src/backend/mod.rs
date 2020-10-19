use broadcaster::BroadcastChannel;
use connected_device::ConnectedDevice;
use state::DeviceState;
use packed_struct::PackedStructSlice;
use crate::{payload::{PayloadConnection, UiPayload}, FractalResult, FractalResultVoid, utils::filter_first, transport::write_struct, transport::write_struct_dyn};
use crate::transport::{Transport, midi::{Midi}, serial::TransportSerial, Endpoint};
use fractal_protocol::{buffer::MessagesBuffer, messages::FractalAudioMessages, messages::effects::Blocks, messages::effects::BlocksHelper, messages::effects::EffectBypassHelper, messages::effects::EffectBypassStatus, messages::effects::EffectStatusHelper, messages::effects::Effects, messages::firmware_version::FirmwareVersionHelper, messages::multipurpose_response::MultipurposeResponseHelper, messages::preset::PresetHelper, messages::scene::SceneWithNameHelper, model::{FractalDevice}};
use std::{time::Duration, thread, pin::Pin};
use log::{error, trace};
use tokio::runtime::Runtime;
use tokio::stream::{pending, Stream};
use futures::{executor::block_on, StreamExt, future::{Either}};
use crate::FractalCoreError;

mod connected_device;
mod state;

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>,
    transports: Vec<Box<dyn Transport>>,
    device: Option<ConnectedDevice>,
    status_poller: Pin<Box<dyn Stream<Item = tokio::time::Instant>>>,
    poll_failures: u8
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
                ],
                poll_failures: 0
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
                    match self.handle_action(msg).await {
                        Err(e) => error!("Error handling the message: {:?}", e),
                        _ => ()
                    }
                },
                PendingAction::EndOfMessagesChannel => {
                    trace!("end of stream!");
                    break;
                },
                PendingAction::Poll => {
                    if let Err(e) = self.status_poll().await {
                        error!("polling failed: {:?}", e);
                    }
                }
            }
        }
    }

    async fn handle_action(&mut self, msg: UiPayload) -> FractalResultVoid {
        match msg {
            UiPayload::Connection(c) => {
                self.connection(c).await?;
            },
            UiPayload::DeviceState(crate::payload::DeviceState::SetPreset{ preset }) => {
                if let Some(ref mut device) = self.device {
                    let bank = (preset / 128) as u8;
                    let patch = (preset % 128) as u8;
                    device.transport_endpoint.write(&vec![0xB0, 0x00, bank])?;
                    device.transport_endpoint.write(&vec![0xC0, patch])?;
                }

                tokio::time::delay_for(Duration::from_millis(10)).await;
                self.status_poll().await?;
            },
            UiPayload::DeviceState(crate::payload::DeviceState::SetScene { scene }) => {
                if let Some(ref mut device) = self.device {
                    device.write(&SceneWithNameHelper::set_current_scene_number(device.device.model, scene))?;
                }

                tokio::time::delay_for(Duration::from_millis(10)).await;
                self.status_poll().await?;
            }
            UiPayload::DeviceState(crate::payload::DeviceState::DeltaPreset { delta }) => {
                if let Some(ref mut device) = self.device {
                    let new_preset = {
                        let p = device.state.preset_number as i16 + delta as i16;
                        if p < 0 { 511 }
                        else if p > 511 { 0 }
                        else { p }
                    };

                    self.send(UiPayload::DeviceState(crate::payload::DeviceState::SetPreset { preset: new_preset as u16 })).await?;
                }
            }
            UiPayload::DeviceState(crate::payload::DeviceState::DeltaScene { delta }) => {
                if let Some(ref mut device) = self.device {
                    let new_scene = {
                        let s = device.state.scene_number as i8 + delta as i8;
                        if s < 0 { 7 }
                        else if s > 7 { 0 }
                        else { s }
                    };

                    self.send(UiPayload::DeviceState(crate::payload::DeviceState::SetScene { scene: new_scene as u8 })).await?;
                }
            }
            UiPayload::RequestAllPresets => {
                
                let presets_count = {
                    let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;
                    device.device.model.number_of_presets().ok_or(FractalCoreError::MissingValue("Number of presets".into()))?
                };
        
                let mut presets = vec![];
                for i in 0..presets_count {
                    let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;
                    let preset = device.send_and_wait_for(&PresetHelper::get_preset_info(device.device.model, i))
                        .await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;
                    presets.push(preset);
                    self.send(UiPayload::ProgressReport { i: i as usize, total: presets_count as usize }).await?;
                }

                self.send(UiPayload::Presets(presets)).await?;
            },
            UiPayload::RequestScenes => {
                let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;
                let scenes_count = device.device.model.number_of_scenes().ok_or(FractalCoreError::MissingValue("Number of scenes".into()))?;

                let mut scenes = vec![];
                for i in 0..scenes_count {
                    let scene = device.send_and_wait_for(&SceneWithNameHelper::get_scene_info(device.device.model, i))
                        .await.map_err(|_| FractalCoreError::MissingValue("Scene".into()))?;
                    scenes.push(scene);
                }

                self.send(UiPayload::Scenes(scenes)).await?;
            },
            UiPayload::RequestCurrentBlocks => {
                let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;

                let blocks: Blocks = device.send_and_wait_for(&BlocksHelper::get_current_blocks(device.device.model))
                                        .await.map_err(|_| FractalCoreError::MissingValue("Blocks".into()))?;
                self.send(UiPayload::CurrentBlocks(blocks)).await?;
            },
            UiPayload::RequestEffectStatus => {
                let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;

                let effects: Effects = device.send_and_wait_for(&EffectStatusHelper::get_status_dump(device.device.model))
                                        .await.map_err(|_| FractalCoreError::MissingValue("Status Dump".into()))?;

                self.send(UiPayload::EffectStatus(effects)).await?;
            }
            UiPayload::SetEffectBypass { effect, is_bypassed } => {
                let device = self.device.as_mut().ok_or(FractalCoreError::NotConnected)?;

                //write_struct_dyn(&mut *device.transport_endpoint, &EffectBypassHelper::set_effect_bypass(device.device.model, effect, is_bypassed))?;
                let effect_status: EffectBypassStatus = device.send_and_wait_for(&EffectBypassHelper::set_effect_bypass(device.device.model, effect, is_bypassed))
                                                              .await.map_err(|_| FractalCoreError::MissingValue("Effect Bypass Status".into()))?;
                self.send(UiPayload::EffectBypassStatus(effect_status)).await?;
            }

            // not for us
            UiPayload::Presets(_) => {}
            UiPayload::Scenes(_) => {}
            UiPayload::CurrentBlocks(_) => {}
            UiPayload::Ping => {}
            UiPayload::Drop => {}
            UiPayload::DeviceState(_) => {}
            UiPayload::EffectStatus(_) => {}
            UiPayload::EffectBypassStatus(_) => {}            
            UiPayload::ProgressReport { .. } => {}
        }

        Ok(())
    }

    async fn send(&self, msg: UiPayload) -> FractalResultVoid {        
        self.channel.send(&msg).await?;
        trace!("Backend sent message: {:?}", msg);
        Ok(())
    }

    fn list_endpoints(&self) -> Vec<Endpoint> {
        Self::list_endpoints_from_transports(&self.transports)
    }

    pub fn list_endpoints_from_transports(transports: &[Box<dyn Transport>]) -> Vec<Endpoint> {
        let mut detected_endpoints = vec![];

        for transport in transports {
            if let Ok(endpoints) = transport.detect_endpoints() {
                for endpoint in endpoints {
                    detected_endpoints.push(Endpoint {
                        transport_id: transport.id().clone(),
                        transport_endpoint: endpoint
                    });
                }
            }
        }

        detected_endpoints
    }

    async fn connection(&mut self, msg: PayloadConnection) -> FractalResultVoid {

        match msg {
            PayloadConnection::ListEndpoints => {
                self.send(UiPayload::Connection(PayloadConnection::DetectedEndpoints {
                    endpoints: self.list_endpoints()
                })).await?;
            },

            PayloadConnection::ConnectToEndpoint(endpoint) => {
                match Self::connect(&endpoint, &self.transports).await {
                    Ok(device) => {                        
                        self.on_connect(device).await?;
                    },
                    Err(e) => {
                        trace!("Connect failed: {:?}", e);
                        self.send(UiPayload::Connection(PayloadConnection::ConnectionFailed(e))).await?
                    }
                }
            },

            PayloadConnection::Disconnect => {
                if let Some(mut device) = self.device.take() {
                    device.transport_endpoint.write(&MultipurposeResponseHelper::disconnect_from_controller(device.device.model).pack_to_vec().unwrap())?;
                }
                
                self.send(UiPayload::Connection(PayloadConnection::Disconnected)).await?;
                self.on_disconnect();                
            }

            PayloadConnection::TryToAutoConnect => {

                let endpoints = self.list_endpoints();
                for endpoint in endpoints {
                    match Self::connect(&endpoint, &self.transports).await {
                        Ok(device) => {
                            self.on_connect(device).await?;
                            return Ok(())
                        },
                        Err(e) => {
                            trace!("Probing failed on port {:?}. Error {:?}", endpoint, e)
                        }
                    }
                }

                self.send(UiPayload::Connection(PayloadConnection::AutoConnectDeviceNotFound)).await?;
            },

            _ => {}
        }

        Ok(())
    }

    /// request the basic infos from the device that might have changed
    async fn status_poll(&mut self) -> FractalResultVoid {
        let max_poll_failures = 5;

        let mut updated = false;

        if let Some(ref mut connected_device) = self.device {
            match connected_device.update_state().await {
                Ok(true) => {
                    updated = true;
                    self.poll_failures = 0;
                },
                Ok(false) => {
                    self.poll_failures = 0;
                },
                Err(e) => {
                    // failed to poll the device.
                    self.poll_failures += 1;
                    error!("Polling failed (attempt {}): {:?}", self.poll_failures, e);

                    // too many polling failures, disconnect.
                    if self.poll_failures >= max_poll_failures {
                        error!("Max poll failures, disconnecting.");
                        self.channel.send(&UiPayload::Connection(PayloadConnection::Disconnect)).await?;
                    }                    
                }
            }
        }

        if updated {
            self.send_device_state().await?;
            self.send(UiPayload::RequestScenes).await?;
            self.send(UiPayload::RequestEffectStatus).await?;
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

    async fn on_connect(&mut self, device: ConnectedDevice) -> FractalResultVoid {
        let info = device.device.clone();
        self.device = Some(device);
        self.send(UiPayload::Connection(PayloadConnection::Connected { device: info })).await?;
        self.send_device_state().await?;
        self.status_poller = Box::pin(tokio::time::interval(Duration::from_millis(1000)));
        self.send(UiPayload::RequestScenes).await?;
        self.send(UiPayload::RequestEffectStatus).await?;
        Ok(())
    }

    fn on_disconnect(&mut self) {
        self.status_poller = Box::pin(pending());
    }

    pub async fn connect(endpoint: &Endpoint, transports: &[Box<dyn Transport>]) -> FractalResult<ConnectedDevice> {
        let timeout = Duration::from_millis(200);
        
        let transport = transports.iter().find(|t| t.id() == endpoint.transport_id).ok_or(FractalCoreError::Other("Transport not found".into()))?;
        let mut connection = transport.connect(&endpoint.transport_endpoint)?;

        let mut midi_messages = BroadcastChannel::<FractalAudioMessages>::new();

        {
            let receiver = connection.get_receiver().clone();
            let midi_messages = midi_messages.clone();

            thread::spawn(move || {
                let mut messages_buffer = MessagesBuffer::new();
                loop {
                    if let Ok(msg) = receiver.recv() {
                        for parsed_msg in messages_buffer.parse(&msg) {
                            trace!("Received SYSEX message: {:?}", parsed_msg);
                            block_on(midi_messages.send(&parsed_msg)).unwrap();
                        }
                    } else {
                        break;
                    }
                }

                trace!("stop bridge");
            });
        }

        // send a message that should reply to us with the model
        connection.write(&MultipurposeResponseHelper::get_discovery_request())?;
        // retrieve the model        
        let model = filter_first(&mut midi_messages, |msg| {
            match msg {
                FractalAudioMessages::MultipurposeResponse(multi_resp)
                    if multi_resp.function_id == 0 && multi_resp.response_code == 0 => 
                {
                    Some(multi_resp.model)
                },
                _ => None
            }
        }, timeout).await;
        let model = model.map_err (|_| FractalCoreError::MissingValue("Model".into()))?;
        trace!("Detected Fractal Model {:?}", model);

        // request the firmware version
        write_struct_dyn(&mut *connection, &FirmwareVersionHelper::get_request(model))?;

        let firmware = filter_first(&mut midi_messages, |msg| {
            match msg {
                FractalAudioMessages::FirmwareVersion(firmware) => {
                    Some(firmware)
                },
                _ => None
            }
        }, timeout).await;
        let firmware = firmware.map_err(|_| FractalCoreError::MissingValue("Firmware".into()))?;
        trace!("Detected firmware {:?}", firmware);

        let device = FractalDevice {
            firmware,
            model
        };

        let mut connected_device = ConnectedDevice {
            device,
            transport_endpoint: connection,
            midi_messages,
            state: DeviceState::default()
        };
        
        match connected_device.update_state().await {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update the state from the freshly connected device: {:?}", e);
            }
        }

        Ok(connected_device)
    }
}