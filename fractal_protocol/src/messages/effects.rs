use std::convert::TryFrom;

use crate::{
    functions::FractalFunction, model::FractalModel,
    structs::FractalAudioMessage,
effect::EffectId, structs::PresetBlockStatus};
use crate::FractalProtocolError;
use super::{parse_sysex_message, MessageHelper};
use crate::packed_struct::PackingError;
use crate::packed_struct::PrimitiveEnum;

type Raw = FractalAudioMessage<Vec<PresetBlockStatus>>;

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


pub struct BlocksHelper;

impl BlocksHelper {
    pub fn get_current_blocks(model: FractalModel) -> FractalAudioMessage<()> {
        FractalAudioMessage::new(model, FractalFunction::GET_SCENE_NAME, ())
    }
}

impl MessageHelper for BlocksHelper {
    type RawResponse = Raw;
    type Response = Blocks;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}

#[derive(Debug, Clone)]
pub struct Effects(Vec<EffectStatus>);

#[derive(Debug, Clone)]
pub struct EffectStatus {
    pub effect_id: EffectId,
    pub supported_channels: u8,
    pub channel: Channel,
    pub is_bypassed: bool
}

#[derive(Copy, Clone, Debug, PrimitiveEnum)]
pub enum Channel {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7
}

pub struct EffectStatusHelper;
impl EffectStatusHelper {
    pub fn get_status_dump(model: FractalModel) -> FractalAudioMessage<()> {
        FractalAudioMessage::new(model, FractalFunction::STATUS_DUMP, ())
    }
}

impl MessageHelper for EffectStatusHelper {
    type RawResponse = FractalAudioMessage<Vec<super::super::structs::EffectStatus>>;
    type Response = Effects;

    fn response_function() -> FractalFunction {
        FractalFunction::STATUS_DUMP
    }
}

impl TryFrom<FractalAudioMessage<Vec<super::super::structs::EffectStatus>>> for Effects {
    type Error = FractalProtocolError;

    fn try_from(value: FractalAudioMessage<Vec<super::super::structs::EffectStatus>>) -> Result<Self, Self::Error> {
        let mut effects = vec![];
        for ef in value.data {
            let effect_id: u16 = ef.effect_id.into();
            let effect_id_typed = EffectId::from_primitive(effect_id as u8);
            let channel: u8 = ef.channel.into();
            let channel_type = Channel::from_primitive(channel);
            let block = EffectStatus {
                effect_id: effect_id_typed.ok_or(FractalProtocolError::UnknownValue { param: "EffectId".into(), value: format!("{:X?}", effect_id)})?,
                channel: channel_type.ok_or(FractalProtocolError::UnknownValue { param: "Channel".into(), value: format!("{:X?}", channel)})?,
                is_bypassed: ef.is_bypassed,
                supported_channels: ef.supported_channels.into()
            };
            effects.push(block);
        }
        Ok(Effects(effects))
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
