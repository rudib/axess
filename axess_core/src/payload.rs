
use fractal_protocol::{effect::EffectId, messages::{effects::Blocks, effects::EffectBypassStatus, effects::Effects, preset::PresetAndName, scene::SceneWithName}};

use crate::transport::{Endpoint};
use crate::FractalCoreError;

#[derive(Debug, Clone)]
pub enum UiPayload {
    Connection(PayloadConnection),
    DeviceState(DeviceState),

    // when connected
    RequestAllPresets,
    Presets(Vec<PresetAndName>),
    RequestScenes,
    Scenes(Vec<SceneWithName>),
    RequestCurrentBlocks,
    CurrentBlocks(Blocks),
    RequestEffectStatus,
    EffectStatus(Effects),
    SetEffectBypass { effect: EffectId, is_bypassed: bool },
    EffectBypassStatus(EffectBypassStatus),

    /// Internal
    Ping,
    
    /// Hard shutdown
    Drop
}

#[derive(Debug, Clone)]
pub enum PayloadConnection {
    ListEndpoints,
    DetectedEndpoints {
        endpoints: Vec<Endpoint>
    },
    ConnectToEndpoint(Endpoint),

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

/*
#[derive(Debug, Clone)]
pub struct ConnectToMidiPorts {
    pub input_port: String,
    pub output_port: String
}
*/
#[derive(Debug, Clone)]
pub enum DeviceState {
    PresetAndScene(PresetAndScene),
    SetPreset { preset: u16 },
    SetScene { scene: u8 }
}

#[derive(Default, Debug, Clone)]
pub struct PresetAndScene {
    pub preset: u16,
    pub preset_name: String,
    pub scene: u8,
    pub scene_name: String
}
/*
#[derive(Default, Debug, Clone)]
pub struct Preset {
    pub preset: u16,
    pub preset_name: String,
}
*/