use packed_struct::{PackedStructSlice, PackingError, PrimitiveEnum};

use crate::{functions::FractalFunction, model::FractalModel};
use super::{FractalHeader, FractalFooter, FractalMessageChecksum};
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

impl<TData> FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    pub fn new(model: FractalModel, func: FractalFunction, data: TData) -> Self {
        let mut msg = Self {
            header: FractalHeader::new(model),
            function: func,
            data: data,
            footer: FractalFooter::default()
        };
        msg.prepare_checksum();
        msg
    }

    pub fn pack(&self) -> Result<Vec<u8>, PackingError> {
        self.pack_to_vec()
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
        let tuple = opt_self.map(|s| {
            (s.header, [0], s.data.clone(), s.footer)
        });
        InternalTuple::<TData>::packed_bytes_size(tuple.as_ref())
    }
}

impl<TData> FractalMessageChecksum for FractalAudioMessage<TData> where TData: PackedStructSlice + Clone {
    fn get_footer(&self) -> &FractalFooter {
        &self.footer
    }

    fn get_footer_mut(&mut self) -> &mut FractalFooter {
        &mut self.footer
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        if let Ok(a) = self.pack_to_vec() {
            a[..a.len() - 2].to_vec()
        } else {
            // todo: change the api?
            vec![]
        }
    }
}

#[test]
fn test_generics() {
    use super::FractalU7;
    use crate::messages::scene::SceneWithNameHelper;

    let msg = FractalAudioMessage::<FractalU7>::new(FractalModel::III, FractalFunction::GET_SCENE_NAME, FractalU7::new_all());
    let packed = msg.pack_to_vec().unwrap();
    println!("{:#X?}", packed);

    let unpacked = FractalAudioMessage::<FractalU7>::unpack_from_slice(&packed).unwrap();
    assert_eq!(msg, unpacked);

    let msg_unit = FractalAudioMessage::<()>::new(FractalModel::FM3, FractalFunction::GET_GRID_LAYOUT_AND_ROUTING, ());
    msg_unit.pack_to_vec().unwrap();

    let sc = SceneWithNameHelper::set_current_scene_number(FractalModel::FM3, 111);
    sc.pack_to_vec().unwrap();
}