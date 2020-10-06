#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(PrimitiveEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum FractalFunction {
    SET_PRESET_NUMBER = 0x3C,
    GET_FIRMWARE_VERSION = 0x08,
    FRONT_PANEL_CHANGE_DETECTED = 0x21,
    DISCONNECT_FROM_CONTROLLER = 0x42,
    GET_MIDI_CHANNEL = 0x17,
    PRESET_INFO = 0x0D,
    //PRESET_BLOCKS_DATA = 0x0E,
    GET_SCENE_NUMBER = 0x29,
    GET_SET_SCENE = 0x0C,
    GET_SET_EFFECT_CHANNEL = 0x0B,
    GET_SET_EFFECT_BYPASS = 0x0A,
    MULTIPURPOSE_RESPONSE = 0x64,
    BATCH_LIST_REQUEST_START = 0x32,
    BATCH_LIST_REQUEST_COMPLETE = 0x33,
    GET_PRESET_NAME = 0x0F,
    GET_SCENE_NAME = 0x0E,
    STATUS_DUMP = 0x13,
    TUNER_ON_OF = 0x11,
    REQUEST_TEMPO = 0x14,
    GET_GRID_LAYOUT_AND_ROUTING = 0x20,
    //GET_CPU_USAGE = 0x13,
    GET_BLOCK_PARAMETERS_LIST = 0x01,
    GET_SET_BLOCK_PARAMETER_VALUE = 0x02,
    SET_TYPED_BLOCK_PARAMETER_VALUE = 0x2E

}

