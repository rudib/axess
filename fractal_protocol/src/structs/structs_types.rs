use packed_struct::prelude::packed_bits;
use packed_struct::{types::Integer};
use crate::{model::FractalModel, buffer::SYSEX_END, buffer::SYSEX_START, buffer::SYSEX_MANUFACTURER};

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalHeader {
    pub sysex_message_start: u8,
    pub sysex_manufacturer: [u8; 3],
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub model: FractalModel
}

impl FractalHeader {
    pub fn new(model: FractalModel) -> Self {
        FractalHeader {            
            model: model,
            ..Default::default()
        }
    }
}

impl Default for FractalHeader {
    fn default() -> Self {
        FractalHeader {
            sysex_message_start: SYSEX_START,
            sysex_manufacturer: SYSEX_MANUFACTURER,
            model: FractalModel::Standard
        }
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalFooter {
    pub checksum: u8,
    pub sysex_message_stop: u8
}

impl Default for FractalFooter {
    fn default() -> Self {
        FractalFooter {
            checksum: 0,
            sysex_message_stop: SYSEX_END
        }
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes="2")]
pub struct FractalU14 {
    #[packed_field(bits="1..")]
    pub lsb: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits="9..")]
    pub msb: Integer<u8, packed_bits::Bits7>
}

impl FractalU14 {
    pub fn new_all() -> Self {
        FractalU14 {
            lsb: 0x7F.into(),
            msb: 0x7F.into()
        }
    }

    pub fn is_all(&self) -> bool {
        *self.lsb == 0x7F && *self.msb == 0x7F
    }
}

impl From<u16> for FractalU14 {
    fn from(n: u16) -> Self {
        Self {
            lsb: ((n & 0x7F) as u8).into(),
            msb: (((n >> 7) & 0x7F) as u8).into()
        }
    }
}

impl Into<u16> for FractalU14 {
    fn into(self) -> u16 {
        (((*self.msb & 0x7F) as u16) << 7) | ((*self.lsb & 0x7F) as u16)
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes="1")]
pub struct FractalU7 {
    #[packed_field(bits="1..")]
    pub msb: Integer<u8, packed_bits::Bits7>
}

impl FractalU7 {
    pub fn new_all() -> Self {
        0x7F.into()
    }
}

impl From<u8> for FractalU7 {
    fn from(n: u8) -> Self {
        FractalU7 {
            msb: n.into()
        }
    }
}

impl Into<u8> for FractalU7 {
    fn into(self) -> u8 {
        self.msb.into()
    }
}

#[derive(Debug, Copy, Clone, PackedStruct)]
pub struct FractalString32 {
    pub data: [u8; 32]
}

impl FractalString32 {
    pub fn from_string(_s: &str) -> Option<Self> {
        panic!("todo");
    }

    pub fn try_as_string(&self) -> Option<String> {
        let len = self.data.iter().take_while(|c| **c != 0 && c.is_ascii()).count();
        let s = String::from_utf8_lossy(&self.data[..len]);
        let s: String = s.into();
        Some(s.trim_end().into())
    }
}

