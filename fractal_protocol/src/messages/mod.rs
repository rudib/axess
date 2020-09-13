pub mod firmware_version;
pub mod multipurpose_response;
pub mod preset;
pub mod scene;

use std::convert::TryFrom;
use preset::{Preset, PresetHelper};
use scene::{Scene, SceneHelper};
use firmware_version::{FirmwareVersion, FirmwareVersionHelper};
use multipurpose_response::{MultipurposeResponse, MultipurposeResponseHelper};

use crate::{structs::{FractalU7, FractalAudioMessage, FractalU14, FractalString32, Data}, functions::FractalFunction, structs::FractalAudioMessageFunction};
use crate::FractalProtocolError;
use crate::packed_struct::PackedStructSlice;

pub trait MessageHelper {
    type RawResponse : packed_struct::PackedStructSlice + FractalAudioMessageFunction;
    type Response : TryFrom<Self::RawResponse> + Into<FractalAudioMessages>;

    fn response_function() -> FractalFunction;
}

#[derive(Debug, Clone)]
pub enum FractalAudioMessages {
    FirmwareVersion(FirmwareVersion),
    MultipurposeResponse(MultipurposeResponse),
    Preset(Preset),
    Scene(Scene)
}

impl From<FirmwareVersion> for FractalAudioMessages {
    fn from(v: FirmwareVersion) -> Self {
        FractalAudioMessages::FirmwareVersion(v)
    }
}

impl From<MultipurposeResponse> for FractalAudioMessages {
    fn from(v: MultipurposeResponse) -> Self {
        FractalAudioMessages::MultipurposeResponse(v)
    }
}

impl From<Preset> for FractalAudioMessages {
    fn from(v: Preset) -> Self {
        FractalAudioMessages::Preset(v)
    }
}

impl From<Scene> for FractalAudioMessages {
    fn from(v: Scene) -> Self {
        FractalAudioMessages::Scene(v)
    }
}

pub fn parse_sysex_message(msg: &[u8]) -> Result<FractalAudioMessages, FractalProtocolError> {
    let mut decoder = SysexDecoder {
        decoded: None
    };

    decoder.try_decode::<PresetHelper>(msg);
    decoder.try_decode::<SceneHelper>(msg);
    decoder.try_decode::<FirmwareVersionHelper>(msg);
    decoder.try_decode::<MultipurposeResponseHelper>(msg);

    decoder.decoded.ok_or(FractalProtocolError::UnknownMessage)
}

struct SysexDecoder {
    decoded: Option<FractalAudioMessages>
}

impl SysexDecoder {
    fn try_decode<T: MessageHelper>(&mut self, msg: &[u8]) {
        if self.decoded.is_some() { return; }

        match T::RawResponse::unpack_from_slice(&msg) {
            Ok(raw) => {
                if raw.get_function() == T::response_function() {
                    match T::Response::try_from(raw) {
                        Ok(decoded) => {
                            self.decoded = Some(decoded.into());
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {}
        }
    }
}