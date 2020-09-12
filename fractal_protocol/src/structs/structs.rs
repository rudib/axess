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


#[derive(Debug, Copy, Clone, PackedStruct, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct FractalMessageEffectStatus {
    pub header: FractalHeader,
    pub function: FractalFunction,
    pub effects: Vec<EffectStatus>,
    pub footer: FractalFooter
}

impl PackedStruct<Vec<u8>> for FractalMessageEffectStatus {
    fn pack(&self) -> Vec<u8> {
        let mut pack = vec![];
        pack.extend_from_slice(&self.header.pack());
        pack.push(self.function.to_primitive());
        for effect in &self.effects {
            pack.extend_from_slice(&effect.pack());
        }
        pack.extend_from_slice(&self.footer.pack());

        pack
    }

    fn unpack(src: &Vec<u8>) -> Result<Self, packed_struct::PackingError> {
        let mut i = 0;
        let n = FractalHeader::packed_bytes();
        let header_slice = &src[i..i+n];        
        let header = FractalHeader::unpack_from_slice(&header_slice)?;
        i += n;

        let function = FractalFunction::from_primitive(src[i]).unwrap();
        i += 1;

        let mut effects = vec![];
        let all_effects_packed = &src[i..(src.len() - 2)];
        let effect_size_bytes = EffectStatus::packed_bytes();
        for effect_packed in all_effects_packed.chunks(effect_size_bytes) {
            let effect = EffectStatus::unpack_from_slice(effect_packed)?;
            effects.push(effect);
        }

        i += all_effects_packed.len();
        
        let footer_slice = &src[i..];
        let footer = FractalFooter::unpack_from_slice(&footer_slice)?;

        Ok(FractalMessageEffectStatus {
            header,
            function,
            effects,
            footer
        })
    }
}

impl FractalMessageChecksum for FractalMessageEffectStatus {
    fn get_footer(&self) -> &FractalFooter {
        &self.footer
    }

    fn get_footer_mut(&mut self) -> &mut FractalFooter {
        &mut self.footer
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len() - 2].to_vec()
    }
}

#[test]
fn test_effect_status_dump() {
    let msg = vec![
        0xf0, 0x00, 0x01, 0x74, 0x10, 0x13, 0x3a, 0x00, 0x40, 0x3e, 0x00, 0x40, 0x2e, 0x00, 0x40,
        0x46, 0x00, 0x40, 0x7a, 0x00, 0x40, 0x3a, 0x01, 0x10, 0x36, 0x01, 0x10, 0x76, 0x00, 0x42,
        0x77, 0x00, 0x41, 0x25, 0x00, 0x40, 0x7e, 0x00, 0x40, 0x2a, 0x00, 0x40, 0x6e, 0x00, 0x41,
        0x32, 0x01, 0x40, 0x42, 0x00, 0x40, 0x43, 0x00, 0x40, 0x6a, 0x00, 0x41, 0x66, 0x00, 0x40,
        0x26, 0x01, 0x10, 0x49, 0x01, 0x10, 0x48, 0x01, 0x10, 0x09, 0xf7,
    ];
    
    let es_packed = [0x3a, 0x00, 0x40];
    let es = EffectStatus::unpack(&es_packed).unwrap();    
    println!("es: {:?}", es);
    println!("{}", es);
    //let effect_id: Option<EffectId> = es.effect_id.into();
    let effect_id: u16 = es.effect_id.into();
    assert_eq!(Some(EffectId::ID_AMP1), EffectId::from_primitive(effect_id as u8));
    assert_eq!(&es_packed, &es.pack());

    let all_effects = FractalMessageEffectStatus::unpack(&msg).unwrap();
    println!("{:?}", all_effects);
    assert!(all_effects.valid_checksum());
}
