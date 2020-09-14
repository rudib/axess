use std::convert::TryFrom;

use super::MessageHelper;
use crate::{FractalProtocolError, model::FractalModel};
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

impl SceneHelper {
    pub fn get_current_scene_info(model: FractalModel) -> FractalAudioMessage<FractalU7> {
        FractalAudioMessage::new(model, FractalFunction::GET_SCENE_NAME, FractalU7::new_all())
    }

    pub fn get_scene_info(model: FractalModel, scene: u8) -> FractalAudioMessage<FractalU7> {
        FractalAudioMessage::new(model, FractalFunction::GET_SCENE_NAME, scene.into())
    }

    pub fn set_current_scene_number(model: FractalModel, scene: u8) -> FractalAudioMessage<FractalU7> {
        FractalAudioMessage::new(model, FractalFunction::GET_SET_SCENE, scene.into())
    }
}

impl MessageHelper for SceneHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU7, FractalString32>>;
    type Response = Scene;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}
