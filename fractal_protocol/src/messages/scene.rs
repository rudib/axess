use std::convert::TryFrom;

use super::MessageHelper;
use crate::{FractalProtocolError, model::FractalModel};
use crate::{
    functions::FractalFunction,
    structs::{FractalAudioMessage, FractalString32, FractalU7},
};
#[derive(Debug, Clone)]
pub struct SceneWithName {
    pub number: u8,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub number: u8
}

impl TryFrom<FractalAudioMessage<(FractalU7, FractalString32)>> for SceneWithName {
    type Error = FractalProtocolError;

    fn try_from(
        value: FractalAudioMessage<(FractalU7, FractalString32)>,
    ) -> Result<Self, Self::Error> {
        Ok(SceneWithName {
            number: value.data.0.into(),
            name: value
                .data
                .1
                .try_as_string()
                .ok_or(FractalProtocolError::MessageConversionError)?,
        })
    }
}

impl TryFrom<FractalAudioMessage<FractalU7>> for Scene {
    type Error = FractalProtocolError;

    fn try_from(
        value: FractalAudioMessage<FractalU7>,
    ) -> Result<Self, Self::Error> {
        Ok(Scene {
            number: value.data.into()
        })
    }
}

pub struct SceneHelper;
pub struct SceneWithNameHelper;

impl SceneWithNameHelper {
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

impl MessageHelper for SceneWithNameHelper {
    type RawResponse = FractalAudioMessage<(FractalU7, FractalString32)>;
    type Response = SceneWithName;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}


impl MessageHelper for SceneHelper {
    type RawResponse = FractalAudioMessage<FractalU7>;
    type Response = Scene;

    fn response_function() -> crate::functions::FractalFunction {
        FractalFunction::GET_SCENE_NAME
    }
}

