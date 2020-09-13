use std::time::Duration;

use broadcaster::BroadcastChannel;
use fractal_protocol::{model::FractalDevice, message::FractalMessageWrapper, message::FractalMessage, commands::Commands, messages::FractalAudioMessages};
use log::trace;

use crate::{transport::TransportConnection, FractalResult, utils::filter_first};
use crate::FractalCoreError;
use super::state::DeviceState;
use crate::packed_struct::PackedStructSlice;

pub struct ConnectedDevice {
    pub transport_endpoint: Box<dyn TransportConnection>,
    pub device: FractalDevice,
    pub midi_messages: BroadcastChannel<FractalAudioMessages>,
    pub state: DeviceState
}


impl ConnectedDevice {
    pub async fn send_and_wait_for<F, T>(&mut self, msg: &[u8], mut filter_map: F) -> FractalResult<T>
        where F: FnMut(&FractalAudioMessages) -> Option<T>
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
        
        let preset = self.send_and_wait_for(&commands.get_current_preset_info().pack_to_vec()?,
|msg| {
                match msg {
                    FractalAudioMessages::Preset(preset) => {
                        Some(preset.clone())
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;

        let scene = self.send_and_wait_for(&commands.get_current_scene_info().pack_to_vec()?, 
|msg| {
                match msg {
                    FractalAudioMessages::Scene(scene) => {
                        Some(scene.clone())
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Scene".into()))?;
        
        let device_state = DeviceState {
            preset_number: preset.number.into(),
            preset_name: preset.name,
            scene_number: scene.number,
            scene_name: scene.name
        };
        
        if device_state != self.state {
            self.state = device_state;
            
            return Ok(true);
        }

        Ok(false)
    }
}
