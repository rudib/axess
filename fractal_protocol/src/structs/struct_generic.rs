use packed_struct::{PackedStructSlice, PackingError, PrimitiveEnum};

use crate::{FractalProtocolError, consts::SYSEX_MANUFACTURER, consts::SYSEX_START, functions::FractalFunction, model::FractalModel, consts::SYSEX_END};
use super::{FractalFooter, FractalHeader, calc_checksum};
#[derive(Debug, Clone, PartialEq)]
pub struct FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    pub header: FractalHeader,
    pub function: FractalFunction,

    pub data: TData,

    pub footer: FractalFooter
}

pub trait FractalAudioMessageFunction {
    fn get_function(&self) -> FractalFunction;
}

impl<TData> FractalAudioMessageFunction for FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    fn get_function(&self) -> FractalFunction {
        self.function
    }
}

pub trait FractalAudioMessagePacker {
    fn pack(&mut self) -> Result<Vec<u8>, PackingError>;
}

pub trait FractalAudioMessageUnpacker where Self: Sized {
    fn unpack_from_slice_with_crc_check(src: &[u8]) -> Result<Self, FractalProtocolError>;
}

impl<TData> FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    pub fn new(model: FractalModel, func: FractalFunction, data: TData) -> Self {
        let msg = Self {
            header: FractalHeader::new(model),
            function: func,
            data,
            footer: FractalFooter::default()
        };

        msg
    }
}

impl<TData> FractalAudioMessagePacker for FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    fn pack(&mut self) -> Result<Vec<u8>, PackingError> {
        let mut packed = self.pack_to_vec()?;
        let len = packed.len();
        let crc = {
            let crc_payload = &packed[..len - 2];
            calc_checksum(crc_payload)
        };
        packed[len - 2] = crc;
        self.footer.checksum = crc;

        Ok(packed)
    }
}

impl<TData> FractalAudioMessageUnpacker for  FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    fn unpack_from_slice_with_crc_check(src: &[u8]) -> Result<Self, FractalProtocolError> {
        let unpacked = Self::unpack_from_slice(src)?;

        // header & footer constant checks
        if unpacked.header.sysex_message_start != SYSEX_START {
            return Err(FractalProtocolError::ConstantMismatch { constant: "sysex_message_start".into() });
        }
        if unpacked.header.sysex_manufacturer != SYSEX_MANUFACTURER {
            return Err(FractalProtocolError::ConstantMismatch { constant: "sysex_message_start".into() });
        }
        if unpacked.footer.sysex_message_stop != SYSEX_END {
            return Err(FractalProtocolError::ConstantMismatch { constant: "sysex_message_stop".into() });
        }

        // check the CRC
        let crc_payload = &src[..src.len() - 2];
        let crc_calculated = calc_checksum(crc_payload);
        let crc_received = unpacked.footer.checksum;
        if crc_calculated != crc_received {
            return Err(FractalProtocolError::CrcMismatch { calculated: crc_calculated, message: crc_received });
        }

        Ok(unpacked)
    }
}

type InternalTuple<T> = (FractalHeader, [u8; 1], T, FractalFooter);
impl<TData> PackedStructSlice for FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    fn pack_to_slice(&self, output: &mut [u8]) -> Result<(), packed_struct::PackingError> {
        let tuple = (self.header, [self.function.to_primitive()], self.data.clone(), self.footer);
        tuple.pack_to_slice(output)
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, packed_struct::PackingError> {
        
        let (header, function, data, footer) = InternalTuple::<TData>::unpack_from_slice(src)?;
        let function = match FractalFunction::from_primitive(function[0]) {
            Some(f) => f,
            None => {
                return Err(packed_struct::PackingError::InvalidValue);
            }
        };
        
        let r = Self {
            header,
            function,
            data,
            footer
        };

        Ok(r)
    }

    fn packed_bytes_size(opt_self: Option<&Self>) -> Result<usize, PackingError> {
        let header = FractalHeader::packed_bytes_size(opt_self.map(|s| &s.header))?;
        let func = 1;
        let data = TData::packed_bytes_size(opt_self.map(|s| &s.data))?;
        let footer = FractalFooter::packed_bytes_size(opt_self.map(|s| &s.footer))?;

        Ok(header + func + data + footer)
    }
}

#[test]
fn test_generics() {
    use super::FractalU7;
    use crate::messages::scene::SceneWithNameHelper;

    let mut msg = FractalAudioMessage::<FractalU7>::new(FractalModel::III, FractalFunction::GET_SCENE_NAME, FractalU7::new_all());
    let packed = msg.pack().unwrap();
    println!("{:#X?}", packed);

    let unpacked = FractalAudioMessage::<FractalU7>::unpack_from_slice(&packed).unwrap();
    assert_eq!(msg, unpacked);

    let mut msg_unit = FractalAudioMessage::<()>::new(FractalModel::FM3, FractalFunction::GET_GRID_LAYOUT_AND_ROUTING, ());
    msg_unit.pack().unwrap();

    let sc = SceneWithNameHelper::set_current_scene_number(FractalModel::FM3, 111);
    sc.pack_to_vec().unwrap();
}