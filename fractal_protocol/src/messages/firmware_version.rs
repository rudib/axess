use std::convert::TryFrom;

use crate::{structs::{FractalAudioMessage, Data, FractalU7}, functions::FractalFunction, model::FractalModel, structs::DataVoid, structs::DataBytes};
use crate::FractalProtocolError;
use super::MessageHelper;

use packed_struct::types::bits::*;

#[derive(Debug, Clone)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8
}

type Raw = FractalAudioMessage<Data<FractalU7, Data<FractalU7, DataBytes<Bytes5>>>>;

impl TryFrom<Raw> for FirmwareVersion {
    type Error = FractalProtocolError;

    fn try_from(value: Raw) -> Result<Self, Self::Error> {
        let (major, minor) = (value.data.0, value.data.1.0);
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
    type RawResponse = Raw;
    type Response = FirmwareVersion;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_FIRMWARE_VERSION
    }
}

#[test]
fn test_parse_resp() {
    use crate::packed_struct::PackedStructSlice;

    let msg = [0xF0, 0x0, 0x1, 0x74, 0x10, 0x8, 0xD, 0x3, 0x10, 0x1, 0x8, 0x0, 0x0, 0xA, 0xF7];
    let s = Raw::packed_bytes();
    println!("s: {}", s);
    println!("msg: {}", msg.len());

    let unpacked = Raw::unpack_from_slice(&msg).unwrap();
    println!("{:?}", unpacked);
}