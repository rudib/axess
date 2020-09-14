use std::convert::TryFrom;

use super::MessageHelper;
use crate::{FractalProtocolError, model::FractalModel};
use crate::{
    functions::FractalFunction,
    structs::{Data, FractalAudioMessage, FractalString32, FractalU14},
};
#[derive(Debug, Clone)]
pub struct PresetAndName {
    pub number: u16,
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct Preset {
    pub number: u16
}

type Raw = FractalAudioMessage<Data<FractalU14, FractalString32>>;

impl TryFrom<Raw> for PresetAndName {
    type Error = FractalProtocolError;

    fn try_from(value: Raw) -> Result<Self, Self::Error> {
        Ok(PresetAndName {
            number: value.data.0.into(),
            name: value
                .data
                .1
                .try_as_string()
                .ok_or(FractalProtocolError::MessageConversionError)?,
        })
    }
}

impl TryFrom<FractalAudioMessage<FractalU14>> for Preset {
    type Error = FractalProtocolError;

    fn try_from(value: FractalAudioMessage<FractalU14>) -> Result<Self, Self::Error> {
        Ok(Preset {
            number: value.data.into()
        })
    }
}

pub struct PresetHelper;

pub struct PresetAndNameHelper;

impl PresetHelper {
    pub fn get_current_preset_info(model: FractalModel) -> FractalAudioMessage<FractalU14> {
        FractalAudioMessage::new(model, FractalFunction::PRESET_INFO, FractalU14::new_all())
    }

    pub fn get_preset_info(model: FractalModel, preset: u16) -> FractalAudioMessage<FractalU14> {
        FractalAudioMessage::new(model, FractalFunction::PRESET_INFO, preset.into())
    }
}

impl MessageHelper for PresetAndNameHelper {
    type RawResponse = Raw;
    type Response = PresetAndName;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::PRESET_INFO
    }
}


impl MessageHelper for PresetHelper {
    type RawResponse = FractalAudioMessage<FractalU14>;
    type Response = Preset;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::PRESET_INFO
    }
}
