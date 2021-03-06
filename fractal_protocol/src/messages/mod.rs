pub mod firmware_version;
pub mod multipurpose_response;
pub mod preset;
pub mod scene;
pub mod effects;

mod tests;

use std::{convert::TryFrom, fmt::Debug};
use effects::{Blocks, BlocksHelper, EffectBypassHelper, EffectBypassStatus, EffectStatusHelper, Effects};
use log::error;
use preset::{Preset, PresetHelper, PresetAndName, PresetAndNameHelper};
use scene::{Scene, SceneHelper, SceneNameRequest, SceneWithName, SceneWithNameHelper, SceneWithNameRequestHelper};
use firmware_version::{FirmwareVersion, FirmwareVersionHelper, FirmwareVersionShortHelper};
use multipurpose_response::{MultipurposeResponse, MultipurposeResponseHelper};

use crate::{functions::FractalFunction, structs::FractalAudioMessageFunction, structs::FractalAudioMessageUnpacker};
use crate::FractalProtocolError;
use crate::packed_struct::PackedStructSlice;

pub trait MessageHelper where <Self::Response as TryFrom<Self::RawResponse>>::Error : Debug {
    type RawResponse : packed_struct::PackedStructSlice + FractalAudioMessageFunction + FractalAudioMessageUnpacker;
    type Response : TryFrom<Self::RawResponse> + Into<FractalAudioMessages>;

    fn response_function() -> FractalFunction;
}

#[derive(Debug, Clone, From, TryInto)]
pub enum FractalAudioMessages {
    FirmwareVersion(FirmwareVersion),
    MultipurposeResponse(MultipurposeResponse),
    PresetAndName(PresetAndName),
    Preset(Preset),
    SceneWithName(SceneWithName),
    Scene(Scene),
    SceneNameRequest(SceneNameRequest),
    Blocks(Blocks),
    Effects(Effects),
    EffectBypassStatus(EffectBypassStatus)
}

pub fn parse_sysex_message(msg: &[u8]) -> Result<FractalAudioMessages, FractalProtocolError> {
    let mut decoder = SysexDecoder {
        decoded: None
    };

    decoder.try_decode::<PresetAndNameHelper>(msg);
    decoder.try_decode::<PresetHelper>(msg);
    decoder.try_decode::<SceneHelper>(msg);
    decoder.try_decode::<SceneWithNameHelper>(msg);
    decoder.try_decode::<SceneWithNameRequestHelper>(msg);
    decoder.try_decode::<FirmwareVersionHelper>(msg);
    decoder.try_decode::<FirmwareVersionShortHelper>(msg);
    decoder.try_decode::<MultipurposeResponseHelper>(msg);
    decoder.try_decode::<BlocksHelper>(msg);
    decoder.try_decode::<EffectStatusHelper>(msg);
    decoder.try_decode::<EffectBypassHelper>(msg);

    decoder.decoded.ok_or(FractalProtocolError::UnknownMessage)
}

struct SysexDecoder {
    decoded: Option<FractalAudioMessages>
}

impl SysexDecoder {
    fn try_decode<T: MessageHelper>(&mut self, msg: &[u8]) where <<T as MessageHelper>::Response as std::convert::TryFrom<<T as MessageHelper>::RawResponse>>::Error: std::fmt::Debug {
        if self.decoded.is_some() { return; }

        match T::RawResponse::unpack_from_slice_with_crc_check(&msg) {
            Ok(raw) => {
                if raw.get_function() == T::response_function() {
                    match T::Response::try_from(raw) {
                        Ok(decoded) => {
                            self.decoded = Some(decoded.into());
                        }
                        Err(e) => {
                            error!("Function {:?} matches, but TryFrom conversion failed: {:?}", T::response_function(), e);
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
}