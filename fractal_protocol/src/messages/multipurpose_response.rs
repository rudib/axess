use std::convert::TryFrom;

use crate::{structs::{FractalAudioMessage, Data, FractalU7}, functions::FractalFunction, model::FractalModel};
use crate::FractalProtocolError;
use super::MessageHelper;

#[derive(Debug, Clone)]
pub struct MultipurposeResponse {
    pub model: FractalModel,
    pub function_id: u8,
    pub response_code: u8
}

impl TryFrom<FractalAudioMessage<Data<FractalU7, FractalU7>>> for MultipurposeResponse {
    type Error = FractalProtocolError;

    fn try_from(value: FractalAudioMessage<Data<FractalU7, FractalU7>>) -> Result<Self, Self::Error> {
        Ok(MultipurposeResponse {
            model: value.header.model,
            function_id: value.data.0.into(),
            response_code: value.data.1.into()
        })
    }
}

pub struct MultipurposeResponseHelper;
impl MessageHelper for MultipurposeResponseHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU7, FractalU7>>;
    type Response = MultipurposeResponse;

    fn response_function() -> FractalFunction {
        FractalFunction::MULTIPURPOSE_RESPONSE
    }
}