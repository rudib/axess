use std::convert::TryFrom;

use super::MessageHelper;
use crate::FractalProtocolError;
use crate::{
    functions::FractalFunction,
    structs::{Data, FractalAudioMessage, FractalString32, FractalU7},
};
#[derive(Debug, Clone)]
pub struct Scene {
    pub number: u8,
    pub name: String,
}

impl TryFrom<FractalAudioMessage<Data<FractalU7, FractalString32>>> for Scene {
    type Error = FractalProtocolError;

    fn try_from(
        value: FractalAudioMessage<Data<FractalU7, FractalString32>>,
    ) -> Result<Self, Self::Error> {
        Ok(Scene {
            number: value.data.0.into(),
            name: value
                .data
                .1
                .try_as_string()
                .ok_or(FractalProtocolError::MessageConversionError)?,
        })
    }
}

pub struct SceneHelper;

impl MessageHelper for SceneHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU7, FractalString32>>;
    type Response = Scene;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}
