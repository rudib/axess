use std::convert::TryInto;

use packed_struct::prelude::packed_bits;
use packed_struct::{
    types::{Integer, ReservedZero}, PackedStructSlice
};

use crate::messages::{effects::Blocks, parse_sysex_message, effects::Effects};

use super::{FractalU14, FractalAudioMessage};
#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "5")]
pub struct PresetBlockStatus {
    pub is_engaged: bool,
    pub x: bool,
    pub _padding: ReservedZero<packed_bits::Bits6>,
    pub bypass_cc: u8,
    pub xy_cc: u8,
    pub effect_id: u8,
    pub _padding2: ReservedZero<packed_bits::Bits3>
}
#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(size_bytes = "3")]
pub struct EffectStatus {
    #[packed_field(element_size_bytes = "2")]
    pub effect_id: FractalU14,
    pub _padding: ReservedZero<packed_bits::Bits1>,
    pub supported_channels: Integer<u8, packed_bits::Bits3>,
    pub channel: Integer<u8, packed_bits::Bits3>,
    pub is_bypassed: bool,
}
