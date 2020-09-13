use std::convert::TryFrom;

use crate::{structs::{FractalAudioMessage, Data, FractalU7}, functions::FractalFunction, model::FractalModel, structs::DataVoid};
use crate::FractalProtocolError;
use super::MessageHelper;

pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8
}

impl TryFrom<FractalAudioMessage<Data<FractalU7, FractalU7>>> for FirmwareVersion {
    type Error = FractalProtocolError;

    fn try_from(value: FractalAudioMessage<Data<FractalU7, FractalU7>>) -> Result<Self, Self::Error> {
        let (major, minor) = (value.data.0, value.data.1);
        Ok(FirmwareVersion {
            major: major.into(),
            minor: minor.into()
        })
    }
}


pub struct FirmwareVersionHelper;

impl FirmwareVersionHelper {
    pub fn get_request(model: FractalModel) -> FractalAudioMessage<DataVoid> {
        FractalAudioMessage::new(model, FractalFunction::GET_FIRMWARE_VERSION, DataVoid)
    }
}

impl MessageHelper for FirmwareVersionHelper {
    type RawResponse = FractalAudioMessage<Data<FractalU7, FractalU7>>;
    type Response = FirmwareVersion;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_FIRMWARE_VERSION
    }
}
