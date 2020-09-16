use std::{time::Duration, convert::TryInto, convert::TryFrom};

use broadcaster::BroadcastChannel;
use fractal_protocol::{model::FractalDevice, messages::FractalAudioMessages, messages::preset::PresetHelper, messages::scene::SceneHelper, messages::scene::SceneWithNameHelper, messages::preset::PresetAndName, messages::scene::SceneWithName};

use crate::{transport::TransportConnection, FractalResult, utils::filter_first, FractalResultVoid};
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
    pub fn write<P: PackedStructSlice>(&mut self, s: &P) -> FractalResultVoid {
        let buf = s.pack_to_vec()?;
        self.transport_endpoint.write(&buf)
    }

    pub async fn send_and_wait_for<T, P: PackedStructSlice>(&mut self, msg: &P) -> FractalResult<T>
        where T: TryInto<T> + TryFrom<FractalAudioMessages>
    {
        let timeout = Duration::from_millis(500);
    
        let mut channel = self.midi_messages.clone();

        self.write(msg)?;

        let r = filter_first(&mut channel, |m| {
            let ms = m.try_into();
            ms.ok()
        }, timeout).await?;

        Ok(r)
    }

    pub async fn update_state(&mut self) -> FractalResult<bool> {
        let preset = self.send_and_wait_for::<PresetAndName>(&PresetHelper::get_current_preset_info(self.device.model))
                    .await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;

        let scene = self.send_and_wait_for::<SceneWithName>(&SceneWithNameHelper::get_current_scene_info(self.device.model))
                    .await.map_err(|_| FractalCoreError::MissingValue("Scene".into()))?;
        
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
