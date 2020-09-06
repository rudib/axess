
use crate::midi::MidiPorts;
use crate::FractalCoreError;

#[derive(Debug, Clone)]
pub enum UiPayload {
    Connection(PayloadConnection),
    DeviceState(DeviceState),

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