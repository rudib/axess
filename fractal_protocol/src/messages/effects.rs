use std::convert::TryFrom;

use crate::{
    functions::FractalFunction, model::FractalModel,
    structs::FractalAudioMessage,
effect::EffectId, structs::EffectStatus};
use crate::FractalProtocolError;
use super::{parse_sysex_message, MessageHelper};
use crate::packed_struct::PackingError;
use crate::packed_struct::PrimitiveEnum;

type Raw = FractalAudioMessage<Vec<EffectStatus>>;

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
        let mut blocks = vec![];
        for ef in value.data {
            let effect_id: u16 = ef.effect_id.into();
            let effect_id_typed = EffectId::from_primitive(effect_id as u8);
            let block = Block {
                effect_id: effect_id_typed.ok_or(FractalProtocolError::UnknownValue { param: "EffectId".into(), value: format!("{:X?}", effect_id)})?
            };
            blocks.push(block);
        }
        Ok(Blocks { blocks })
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
        FractalFunction::STATUS_DUMP
    }
}

#[test]
fn test_parse_blocks() {
    let raw = [0xf0, 0x00, 0x01, 0x74, 0x10, 0x13, 0x3a, 0x00, 0x40, 0x3e, 0x00, 0x40, 0x2e, 0x00, 0x40,
    0x46, 0x00, 0x40, 0x7a, 0x00, 0x40, 0x3a, 0x01, 0x10, 0x36, 0x01, 0x10, 0x76, 0x00, 0x42,
    0x77, 0x00, 0x41, 0x25, 0x00, 0x40, 0x7e, 0x00, 0x40, 0x2a, 0x00, 0x40, 0x6e, 0x00, 0x41,
    0x32, 0x01, 0x40, 0x42, 0x00, 0x40, 0x43, 0x00, 0x40, 0x6a, 0x00, 0x41, 0x66, 0x00, 0x40,
    0x26, 0x01, 0x10, 0x49, 0x01, 0x10, 0x48, 0x01, 0x10, 0x09, 0xf7];
    let msg = parse_sysex_message(&raw).unwrap();
    println!("msg: {:?}", msg);
}
