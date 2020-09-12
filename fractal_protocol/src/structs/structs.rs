use packed_struct::{prelude::packed_bits, types::ReservedZero};
use packed_struct::{types::Integer, PackedStruct};
use packed_struct::types::SizedInteger;
use packed_struct::types::bits::*;
use packed_struct::{PrimitiveEnum, PackedStructSlice};
//use packed_struct::prelude::*;

use crate::{
    effect::EffectId, functions::FractalFunction, message2::SYSEX_END, message2::SYSEX_HEADER,
    message2::SYSEX_MANUFACTURER, message2::SYSEX_START, model::FractalModel};

use super::{FractalHeader, FractalFooter, FractalMessageChecksum, FractalU14, FractalU7, EffectStatus};


