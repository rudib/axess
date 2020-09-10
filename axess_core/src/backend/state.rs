
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct DeviceState {
    pub preset_number: u32,
    pub preset_name: String,
    pub scene_number: u8,
    pub scene_name: String
}