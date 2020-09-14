pub mod firmware_version;
pub mod multipurpose_response;
pub mod preset;
pub mod scene;

use std::{convert::TryFrom, fmt::Debug};
use log::error;
use preset::{Preset, PresetHelper, PresetAndName, PresetAndNameHelper};
use scene::{SceneWithName, SceneHelper, Scene, SceneWithNameHelper};
use firmware_version::{FirmwareVersion, FirmwareVersionHelper};
use multipurpose_response::{MultipurposeResponse, MultipurposeResponseHelper};

use crate::{functions::FractalFunction, structs::FractalAudioMessageFunction};
use crate::FractalProtocolError;
use crate::packed_struct::PackedStructSlice;

pub trait MessageHelper where <Self::Response as TryFrom<Self::RawResponse>>::Error : Debug {
    type RawResponse : packed_struct::PackedStructSlice + FractalAudioMessageFunction;
    type Response : TryFrom<Self::RawResponse> + Into<FractalAudioMessages>;

    fn response_function() -> FractalFunction;
}

#[derive(Debug, Clone)]
pub enum FractalAudioMessages {
    FirmwareVersion(FirmwareVersion),
    MultipurposeResponse(MultipurposeResponse),
    PresetAndName(PresetAndName),
    Preset(Preset),
    SceneWithName(SceneWithName),
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

impl From<PresetAndName> for FractalAudioMessages {
    fn from(v: PresetAndName) -> Self {
        FractalAudioMessages::PresetAndName(v)
    }
}

impl From<SceneWithName> for FractalAudioMessages {
    fn from(v: SceneWithName) -> Self {
        FractalAudioMessages::SceneWithName(v)
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

    decoder.try_decode::<PresetAndNameHelper>(msg);
    decoder.try_decode::<PresetHelper>(msg);
    decoder.try_decode::<SceneHelper>(msg);
    decoder.try_decode::<SceneWithNameHelper>(msg);
    decoder.try_decode::<FirmwareVersionHelper>(msg);
    decoder.try_decode::<MultipurposeResponseHelper>(msg);

    decoder.decoded.ok_or(FractalProtocolError::UnknownMessage)
}

struct SysexDecoder {
    decoded: Option<FractalAudioMessages>
}

impl SysexDecoder {
    fn try_decode<T: MessageHelper>(&mut self, msg: &[u8]) where <<T as MessageHelper>::Response as std::convert::TryFrom<<T as MessageHelper>::RawResponse>>::Error: std::fmt::Debug {
        if self.decoded.is_some() { return; }

        match T::RawResponse::unpack_from_slice(&msg) {
            Ok(raw) => {
                println!("function: {:?}", raw.get_function());
                if raw.get_function() == T::response_function() {
                    println!("function match");
                    match T::Response::try_from(raw) {
                        Ok(decoded) => {
                            self.decoded = Some(decoded.into());
                        }
                        Err(e) => {
                            error!("Function {:?} matches, but TryFrom conversion failed: {:?}", T::response_function(), e);
                            println!("Function {:?} matches, but TryFrom conversion failed: {:?}", T::response_function(), e);
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
}