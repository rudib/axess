use axess_core::payload::PresetAndScene;
use fractal_protocol::messages::{effects::Effects, preset::PresetAndName, scene::SceneWithName};


/// Holds all the latest information about the device that we know.
#[derive(Default, Debug)]
pub struct FrontendDeviceState {
    pub current_preset_and_scene: Option<PresetAndScene>,
    pub presets: Vec<PresetAndName>,
    pub current_presets_scenes: Vec<SceneWithName>,
    pub current_effects: Option<Effects>
}