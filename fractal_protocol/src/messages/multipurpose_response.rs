use std::convert::TryFrom;

use crate::{structs::{FractalAudioMessage, Data, FractalU7}, functions::FractalFunction};
use crate::FractalProtocolError;
use super::MessageHelper;


pub struct MultipurposeResponse {
    pub response_code: u8
}

impl TryFrom<FractalAudioMessage<FractalU7>> for MultipurposeResponse {
    type Error = FractalProtocolError;

    fn try_from(value: FractalAudioMessage<FractalU7>) -> Result<Self, Self::Error> {
        Ok(MultipurposeResponse {
            response_code: value.data.into()
        })
    }
}

pub struct MultipurposeResponseHelper;
impl MessageHelper for MultipurposeResponseHelper {
    type RawResponse = FractalAudioMessage<FractalU7>;
    type Response = MultipurposeResponse;

    fn response_function() -> FractalFunction {
        FractalFunction::MULTIPURPOSE_RESPONSE
    }
}