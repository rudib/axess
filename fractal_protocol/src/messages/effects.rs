use std::convert::TryFrom;

use crate::{
    functions::FractalFunction, model::FractalModel,
    structs::FractalAudioMessage,
effect::EffectId, structs::EffectStatus};
use crate::FractalProtocolError;
use super::{parse_sysex_message, MessageHelper};

type Raw = FractalAudioMessage<EffectStatus>;

#[derive(Clone, Debug)]
pub struct Blocks {
    pub blocks: Vec<Block>
}

#[derive(Clone, Debug)]
pub struct Block {
    pub effect_id: EffectId
}


impl TryFrom<Raw> for Blocks {
    type Error = FractalProtocolError;

    fn try_from(value: Raw) -> Result<Self, Self::Error> {
        todo!()
    }
}


pub struct EffectsHelper;

impl EffectsHelper {
    pub fn get_current_blocks(model: FractalModel) -> FractalAudioMessage<()> {
        FractalAudioMessage::new(model, FractalFunction::GET_SCENE_NAME, ())
    }
}

impl MessageHelper for EffectsHelper {
    type RawResponse = Raw;
    type Response = Blocks;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}

#[test]
fn test_parse_blocks() {
    let raw = [
        0xF0, 0x0, 0x1, 0x74, 0x10, 0xE, 0x1B, 0x0, 0x0, 0x0, 0x3F, 0x5D, 0x4D, 0x2E, 0x3F, 0x0,
        0x0, 0x3F, 0x0, 0x0, 0x3F, 0x3E, 0x0, 0x0, 0x0, 0x0, 0x4D, 0x26, 0x3F, 0x73, 0x3E, 0x36,
        0xF7,
    ];

    let msg = parse_sysex_message(&raw).unwrap();
    println!("msg: {:?}", msg);
}
