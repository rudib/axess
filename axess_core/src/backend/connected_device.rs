use std::time::Duration;

use broadcaster::BroadcastChannel;
use fractal_protocol::{model::FractalDevice, message::FractalMessageWrapper, message::FractalMessage, common::get_current_preset_name, common::get_current_scene_name};

use crate::{transport::TransportConnection, FractalResult, utils::filter_first};
use crate::FractalCoreError;
use super::state::DeviceState;

pub struct ConnectedDevice {
    pub transport_endpoint: Box<dyn TransportConnection>,
    pub device: FractalDevice,
    pub midi_messages: BroadcastChannel<FractalMessageWrapper>,
    pub state: DeviceState
}


impl ConnectedDevice {
    pub async fn send_and_wait_for<F, T>(&mut self, msg: &[u8], mut filter_map: F) -> FractalResult<T>
        where F: FnMut(&FractalMessageWrapper) -> Option<T>
    {
        let timeout = Duration::from_millis(500);
    
        let mut channel = self.midi_messages.clone();

        self.transport_endpoint.write(msg)?;

        let r = filter_first(&mut channel, |m| {
            filter_map(&m)
        }, timeout).await?;

        Ok(r)
    }

    pub async fn update_state(&mut self) -> FractalResult<bool> {
        let (preset_number, preset_name) = self.send_and_wait_for(&get_current_preset_name(self.device.model),
|msg| {
                match &msg.message {
                    FractalMessage::PresetName(preset_number, preset_name) => {
                        Some((*preset_number,preset_name.clone()))
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;

        let (scene_number, scene_name) = self.send_and_wait_for(&get_current_scene_name(self.device.model), 
|msg| {
                match &msg.message {
                    FractalMessage::SceneName(scene, name) => {
                        Some((*scene, name.clone()))
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Scene".into()))?;
        
        let device_state = DeviceState {
            preset_number,
            preset_name,
            scene_number,
            scene_name
        };
        
        if device_state != self.state {
            self.state = device_state;
            
            return Ok(true);
        }

        Ok(false)
    }
}