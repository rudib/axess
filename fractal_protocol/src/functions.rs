
#[derive(PrimitiveEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum FractalFunction {
    GET_CURRENT_PRESET_NUMBER = 0x14,
    SET_PRESET_NUMBER = 0x3C,
    GET_FIRMWARE_VERSION = 0x08,
    FRONT_PANEL_CHANGE_DETECTED = 0x21,
    DISCONNECT_FROM_CONTROLLER = 0x42,
    GET_MIDI_CHANNEL = 0x17,
    TUNER_INFO = 0x0D,
    PRESET_BLOCKS_DATA = 0x0E,
    GET_SCENE_NUMBER = 0x29,
    GET_BLOCK_PARAMETERS_LIST = 0x01,
    MULTIPURPOSE_RESPONSE = 0x64,
    BATCH_LIST_REQUEST_START = 0x32,
    BATCH_LIST_REQUEST_COMPLETE = 0x33,
    GET_PRESET_NAME = 0x0F,
    //GET_SCENE_NAME = 0x0E
}

