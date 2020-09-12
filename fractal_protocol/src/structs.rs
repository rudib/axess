use packed_struct::{prelude::packed_bits, types::ReservedZero};
use packed_struct::{types::Integer, PackedStruct};
use packed_struct::types::SizedInteger;
use packed_struct::types::bits::*;
use packed_struct::PrimitiveEnum;
//use packed_struct::prelude::*;

use crate::{
    effect::EffectId, functions::FractalFunction, message2::SYSEX_END, message2::SYSEX_HEADER,
    message2::SYSEX_MANUFACTURER, message2::SYSEX_START, model::FractalModel,
structs_types::FractalHeader, structs_types::FractalFooter, structs_types::FractalU14, structs_types::FractalU7};



#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmd {
    #[packed_field(element_size_bytes="5")]
    header: FractalHeader,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    #[packed_field(element_size_bytes="2")]
    footer: FractalFooter
}

impl FractalCmd {
    pub fn new(model: FractalModel, function: FractalFunction) -> Self {
        let mut cmd = FractalCmd {
            header: FractalHeader::new(model),
            function: function,
            footer: FractalFooter::default()
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmd {
    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }

    fn get_footer(&self) -> &FractalFooter {
        &self.footer
    }

    fn get_footer_mut(&mut self) -> &mut FractalFooter {
        &mut self.footer
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq)]
pub struct FractalCmdWithU14 {
    #[packed_field(element_size_bytes="5")]
    header: FractalHeader,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    #[packed_field(element_size_bytes = "2")]
    pub data: FractalU14,
    #[packed_field(element_size_bytes="2")]
    footer: FractalFooter
}

impl FractalCmdWithU14 {
    pub fn new(model: FractalModel, function: FractalFunction, data: FractalU14) -> Self {
        let mut cmd = Self {
            header: FractalHeader::new(model),
            function: function,
            data,
            footer: FractalFooter::default()
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmdWithU14 {
    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }

    fn get_footer(&self) -> &FractalFooter {
        &self.footer
    }

    fn get_footer_mut(&mut self) -> &mut FractalFooter {
        &mut self.footer
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq)]
pub struct FractalCmdWithU7 {
    #[packed_field(element_size_bytes="5")]
    pub header: FractalHeader,
    #[packed_field(element_size_bytes = "1", ty = "enum")]
    pub function: FractalFunction,
    #[packed_field(element_size_bytes="1")]
    pub data: FractalU7,
    #[packed_field(element_size_bytes="2")]
    pub footer: FractalFooter
    
}

impl FractalCmdWithU7 {
    pub fn new(model: FractalModel, function: FractalFunction, data: FractalU7) -> Self {
        let mut cmd = FractalCmdWithU7 {
            header: FractalHeader::new(model),
            function: function,
            data: data.into(),
            footer: FractalFooter::default()
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmdWithU7 {
    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }

    fn get_footer(&self) -> &FractalFooter {
        &self.footer
    }

    fn get_footer_mut(&mut self) -> &mut FractalFooter {
        &mut self.footer
    }
}

pub trait FractalMessageChecksum {
    fn get_footer(&self) -> &FractalFooter;
    fn get_footer_mut(&mut self) -> &mut FractalFooter;
    fn get_checksum_payload(&self) -> Vec<u8>;

    fn prepare_checksum(&mut self) {
        let crc = calc_checksum(&self.get_checksum_payload());
        let mut footer = self.get_footer_mut();
        footer.checksum = crc;
    }

    fn valid_checksum(&self) -> bool {
        let crc_calculated = calc_checksum(&self.get_checksum_payload());
        self.get_footer().checksum == crc_calculated
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

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(bit_numbering = "msb0", size_bytes="3")]
pub struct EffectStatus {
    #[packed_field(element_size_bytes = "2")]
    pub effect_id: FractalU14,
    pub _padding: ReservedZero<packed_bits::Bits1>,
    pub supported_channels: Integer<u8, packed_bits::Bits3>,
    pub channel: Integer<u8, packed_bits::Bits3>,
    pub is_bypassed: bool
}


#[test]
fn test_pack_cmd_with_int() {
    let msg = FractalCmdWithU7::new(FractalModel::III, FractalFunction::GET_SCENE_NAME, 0x7F.into());
    println!("{:?}", msg);
    let packed = msg.pack();
    println!("packed: {:#X?}", packed);

    let unpacked = FractalCmdWithU7::unpack(&packed).unwrap();
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
    //let effect_id: Option<EffectId> = es.effect_id.into();
    let effect_id: u16 = es.effect_id.into();
    assert_eq!(Some(EffectId::ID_AMP1), EffectId::from_primitive(effect_id as u8));
    assert_eq!(&es_packed, &es.pack());
}
