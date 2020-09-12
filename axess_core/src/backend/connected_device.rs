use std::time::Duration;

use broadcaster::BroadcastChannel;
use fractal_protocol::{model::FractalDevice, message::FractalMessageWrapper, message::FractalMessage, common::get_current_preset_name, common::get_current_scene_name, structs::FractalCmd, functions::FractalFunction, structs::FractalCmdWithU14, structs::FractalCmdWithU7, structs_types::FractalU14, structs_types::FractalU7, commands::Commands};
use log::trace;

use crate::{transport::TransportConnection, FractalResult, utils::filter_first};
use crate::FractalCoreError;
use super::state::DeviceState;
use crate::packed_struct::PackedStruct;

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
        let commands = Commands::new(self.device.model);
        
        let (preset_number, preset_name) = self.send_and_wait_for(&commands.get_current_preset_info().pack(),
|msg| {
                match &msg.message {
                    FractalMessage::PresetName(preset_number, preset_name) => {
                        Some((*preset_number,preset_name.clone()))
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;

        let (scene_number, scene_name) = self.send_and_wait_for(&commands.get_current_scene_info().pack(), 
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
