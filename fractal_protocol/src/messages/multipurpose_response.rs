use std::convert::TryFrom;

use crate::{structs::{FractalAudioMessage, Data, FractalU7}, functions::FractalFunction, model::FractalModel, structs::DataVoid};
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

impl MultipurposeResponseHelper {
    pub fn get_discovery_request() -> Vec<u8> {
        vec![0xF0, 0x0, 0x1, 0x74, 0x7F, 0x0, 0x7A, 0xF7]
    }

    pub fn disconnect_from_controller(model: FractalModel) -> FractalAudioMessage<DataVoid> {
        FractalAudioMessage::new(model, FractalFunction::DISCONNECT_FROM_CONTROLLER, DataVoid)
    }
}

impl MessageHelper for MultipurposeResponseHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU7, FractalU7>>;
    type Response = MultipurposeResponse;

    fn response_function() -> FractalFunction {
        FractalFunction::MULTIPURPOSE_RESPONSE
    }
}