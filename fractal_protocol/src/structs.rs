use packed_struct::prelude::packed_bits;
use packed_struct::{types::Integer, PackedStruct};
use packed_struct::types::SizedInteger;
use packed_struct::types::bits::*;
use packed_struct::PrimitiveEnum;
//use packed_struct::prelude::*;

use crate::{
    effect::EffectId, functions::FractalFunction, message2::SYSEX_END, message2::SYSEX_HEADER,
    message2::SYSEX_MANUFACTURER, message2::SYSEX_START, model::FractalModel,
};

/*
#[derive(PackedStruct)]
#[packed_struct(bit_numbering="msb0")]
pub struct TestPack {
    #[packed_field(bits="0..=2")]
    tiny_int: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="3..=4", ty="enum")]
    mode: SelfTestMode,
    #[packed_field(bits="7")]
    enabled: bool
}
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FractalInt {
    Int(u16),
    All,
}

//pub trait FractalCmdBase {
//    fn init(model: FractalModel) -> Self;
//}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmd {
    pub sysex_message_start: u8,
    pub manufacturer: [u8; 3],
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub model: FractalModel,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    pub checksum: u8,
    pub sysex_message_stop: u8,
}

impl FractalCmd {
    pub fn new(model: FractalModel, function: FractalFunction) -> Self {
        let mut cmd = FractalCmd {
            sysex_message_start: SYSEX_START,
            manufacturer: SYSEX_MANUFACTURER,
            model: model,
            function: function,
            checksum: 0,
            sysex_message_stop: SYSEX_END,
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmd {
    fn get_checksum(&self) -> u8 {
        self.checksum
    }

    fn set_checksum(&mut self, checksum: u8) {
        self.checksum = checksum
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmdWithInt {
    pub sysex_message_start: u8,
    pub manufacturer: [u8; 3],
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub model: FractalModel,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    #[packed_field(element_size_bytes = "2")]
    pub int: FractalInt,
    pub checksum: u8,
    pub sysex_message_stop: u8,
}

impl FractalCmdWithInt {
    pub fn new(model: FractalModel, function: FractalFunction, int: FractalInt) -> Self {
        let mut cmd = FractalCmdWithInt {
            sysex_message_start: SYSEX_START,
            manufacturer: SYSEX_MANUFACTURER,
            model: model,
            function: function,
            int,
            checksum: 0,
            sysex_message_stop: SYSEX_END,
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmdWithInt {
    fn get_checksum(&self) -> u8 {
        self.checksum
    }

    fn set_checksum(&mut self, checksum: u8) {
        self.checksum = checksum
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmdWithByte {
    pub sysex_message_start: u8,
    pub manufacturer: [u8; 3],
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub model: FractalModel,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    pub data: u8,
    pub checksum: u8,
    pub sysex_message_stop: u8,
}

impl FractalCmdWithByte {
    pub fn new(model: FractalModel, function: FractalFunction, data: u8) -> Self {
        let mut cmd = FractalCmdWithByte {
            sysex_message_start: SYSEX_START,
            manufacturer: SYSEX_MANUFACTURER,
            model: model,
            function: function,
            data,
            checksum: 0,
            sysex_message_stop: SYSEX_END,
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmdWithByte {
    fn get_checksum(&self) -> u8 {
        self.checksum
    }

    fn set_checksum(&mut self, checksum: u8) {
        self.checksum = checksum
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }
}

pub trait FractalMessageChecksum {
    fn get_checksum(&self) -> u8;
    fn set_checksum(&mut self, checksum: u8);
    fn get_checksum_payload(&self) -> Vec<u8>;

    fn prepare_checksum(&mut self) {
        let crc = calc_checksum(&self.get_checksum_payload());
        self.set_checksum(crc);
    }

    fn valid_checksum(&self) -> bool {
        let crc_calculated = calc_checksum(&self.get_checksum_payload());
        self.get_checksum() == crc_calculated
    }
}

fn calc_checksum(sysex: &[u8]) -> u8 {
    if sysex.len() < 2 {
        return 0;
    }

    let mut sum = sysex[0];
    for b in &sysex[1..] {
        sum ^= *b;
    }
    sum & 0x7F
}

impl PackedStruct<[u8; 2]> for FractalInt {
    fn pack(&self) -> [u8; 2] {
        match self {
            FractalInt::Int(n) => [(n & 0x7F) as u8, ((n >> 7) & 0x7F) as u8],
            FractalInt::All => [0x7F, 0x7F],
        }
    }

    fn unpack(src: &[u8; 2]) -> Result<Self, packed_struct::PackingError> {
        if src == &[0x7F, 0x7F] {
            Ok(FractalInt::All)
        } else {
            Ok(FractalInt::Int(
                ((src[0] & 0x7F) as u16) | (((src[1] & 0x7F) as u16) << 7),
            ))
        }
    }
}

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(bit_numbering = "msb0", size_bytes="3")]
pub struct EffectStatus {
    #[packed_field(element_size_bytes = "2", bits = "0..")]
    pub effect_id: StatusEffectId,
    #[packed_field(bits = "17..")]
    pub supported_channels: Integer<u8, packed_bits::Bits3>,
    pub channel: Integer<u8, packed_bits::Bits3>,
    pub is_bypassed: bool,
}

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(bit_numbering = "msb0", size_bytes="2")]
pub struct StatusEffectId {
    #[packed_field(bits="1..")]
    pub lsb: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits="8..")]
    pub msb: Integer<u8, packed_bits::Bits7>
}

impl From<u16> for StatusEffectId {
    fn from(n: u16) -> Self {
        Self {
            lsb: (n as u8).into(),
            msb: ((n << 7) as u8).into()
        }
    }
}

impl Into<u16> for StatusEffectId {
    fn into(self) -> u16 {
        (*self.lsb as u16) | ((*self.msb as u16) << 7)
    }
}

impl Into<Option<EffectId>> for StatusEffectId {
    fn into(self) -> Option<EffectId> {
        let n: u16 = self.into();
        EffectId::from_primitive(n as u8)
    }
}

#[test]
fn test_pack_cmd_with_int() {
    let mut msg = FractalCmdWithInt {
        sysex_message_start: SYSEX_START,
        manufacturer: SYSEX_MANUFACTURER,
        model: FractalModel::III,
        function: FractalFunction::GET_PRESET_NAME,
        int: FractalInt::Int(150),
        checksum: 0,
        sysex_message_stop: SYSEX_END,
    };
    msg.prepare_checksum();
    println!("{:?}", msg);
    let packed = msg.pack();
    println!("packed: {:#X?}", packed);

    let unpacked = FractalCmdWithInt::unpack(&packed).unwrap();
    assert!(unpacked.valid_checksum());
    assert_eq!(msg, unpacked);
}

/*
let msg = [
    0xf0, 0x00, 0x01, 0x74, 0x10, 0x01, 0x1a, 0x00, 0x02, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2c, 0xf7,
];
*/

#[test]
fn test_effect_status_dump() {
    let msg = vec![
        0xf0, 0x00, 0x01, 0x74, 0x10, 0x13, 0x3a, 0x00, 0x40, 0x3e, 0x00, 0x40, 0x2e, 0x00, 0x40,
        0x46, 0x00, 0x40, 0x7a, 0x00, 0x40, 0x3a, 0x01, 0x10, 0x36, 0x01, 0x10, 0x76, 0x00, 0x42,
        0x77, 0x00, 0x41, 0x25, 0x00, 0x40, 0x7e, 0x00, 0x40, 0x2a, 0x00, 0x40, 0x6e, 0x00, 0x41,
        0x32, 0x01, 0x40, 0x42, 0x00, 0x40, 0x43, 0x00, 0x40, 0x6a, 0x00, 0x41, 0x66, 0x00, 0x40,
        0x26, 0x01, 0x10, 0x49, 0x01, 0x10, 0x48, 0x01, 0x10, 0x09, 0xf7,
    ];

    /*
    let es = EffectStatus {
        channel: 0.into(),
        effect_id: StatusEffectId(EffectId::ID_DISTORT1),
        supported_channels: 0.into(),
        is_bypassed: false,
    };
    println!("{}", es);
    */

    let es_packed = [0x3a, 0x00, 0x40];
    let es = EffectStatus::unpack(&es_packed).unwrap();    
    println!("es: {:?}", es);
    println!("{}", es);
    let effect_id: Option<EffectId> = es.effect_id.into();
    assert_eq!(Some(EffectId::ID_AMP1), effect_id);
    assert_eq!(&es_packed, &es.pack());
}
