use std::convert::{TryFrom, TryInto};

use super::{parse_sysex_message, MessageHelper};
use crate::FractalProtocolError;
use crate::{
    functions::FractalFunction,
    model::FractalModel,
    structs::DataBytes,
    structs::{FractalAudioMessage, FractalU7},
};

use packed_struct::types::bits::*;

#[derive(Debug, Clone)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
}

type Raw = FractalAudioMessage<(FractalU7, (FractalU7, DataBytes<Bytes5>))>;
type RawShort = FractalAudioMessage<(FractalU7, (FractalU7, DataBytes<Bytes2>))>;

impl TryFrom<Raw> for FirmwareVersion {
    type Error = FractalProtocolError;

    fn try_from(value: Raw) -> Result<Self, Self::Error> {
        let (major, minor) = (value.data.0, (value.data.1).0);
        Ok(FirmwareVersion {
            major: major.into(),
            minor: minor.into(),
        })
    }
}

impl TryFrom<RawShort> for FirmwareVersion {
    type Error = FractalProtocolError;

    fn try_from(value: RawShort) -> Result<Self, Self::Error> {
        let (major, minor) = (value.data.0, (value.data.1).0);
        Ok(FirmwareVersion {
            major: major.into(),
            minor: minor.into(),
        })
    }
}

pub struct FirmwareVersionHelper;

impl FirmwareVersionHelper {
    pub fn get_request(model: FractalModel) -> FractalAudioMessage<()> {
        FractalAudioMessage::new(model, FractalFunction::GET_FIRMWARE_VERSION, ())
    }
}


impl MessageHelper for FirmwareVersionHelper {
    type RawResponse = Raw;
    type Response = FirmwareVersion;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_FIRMWARE_VERSION
    }
}

pub struct FirmwareVersionShortHelper;

impl MessageHelper for FirmwareVersionShortHelper {
    type RawResponse = RawShort;
    type Response = FirmwareVersion;

    fn response_function() -> FractalFunction {
        FractalFunction::GET_FIRMWARE_VERSION
    }
}

#[test]
fn test_parse_resp_axe3() {
    let msg = [
        0xF0, 0x0, 0x1, 0x74, 0x10, 0x8, 0xD, 0x3, 0x10, 0x1, 0x8, 0x0, 0x0, 0xA, 0xF7,
    ];
    
    let decoded = parse_sysex_message(&msg).unwrap();
    let msg: FirmwareVersion = decoded.try_into().unwrap();
    assert_eq!(13, msg.major);
    assert_eq!(3, msg.minor);
}

#[test]
fn test_parse_resp_fm3() {
    let raw = [
        0xF0, 0x0, 0x1, 0x74, 0x11, 0x8, 0x1, 0x5, 0x0, 0x0, 0x18, 0xF7,
    ];

    let decoded = parse_sysex_message(&raw).unwrap();
    let msg: FirmwareVersion = decoded.try_into().unwrap();
    assert_eq!(1, msg.major);
    assert_eq!(5, msg.minor);
}
