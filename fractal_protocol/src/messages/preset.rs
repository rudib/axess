use std::convert::TryFrom;

use super::MessageHelper;
use crate::FractalProtocolError;
use crate::{
    functions::FractalFunction,
    structs::{Data, FractalAudioMessage, FractalString32, FractalU14},
};
#[derive(Debug, Clone)]
pub struct Preset {
    pub number: u16,
    pub name: String,
}

impl TryFrom<FractalAudioMessage<Data<FractalU14, FractalString32>>> for Preset {
    type Error = FractalProtocolError;

    fn try_from(
        value: FractalAudioMessage<Data<FractalU14, FractalString32>>,
    ) -> Result<Self, Self::Error> {
        Ok(Preset {
            number: value.data.0.into(),
            name: value
                .data
                .1
                .try_as_string()
                .ok_or(FractalProtocolError::MessageConversionError)?,
        })
    }
}

pub struct PresetHelper;

impl MessageHelper for PresetHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU14, FractalString32>>;
    type Response = Preset;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::PRESET_INFO
    }
}
