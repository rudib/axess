use packed_struct::{PackedStruct, PackedStructSlice, PackingError, PrimitiveEnum};

use crate::{functions::FractalFunction, model::FractalModel};
use super::{FractalHeader, FractalFooter, FractalMessageChecksum};
#[derive(Debug, Clone, PartialEq)]
pub struct FractalAudioMessage<TData> {
    pub header: FractalHeader,
    pub function: FractalFunction,

    pub data: TData,

    pub footer: FractalFooter
}

impl<TData> FractalAudioMessage<TData> where TData: PackedStructSlice {
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
}

impl<TData> PackedStructSlice for FractalAudioMessage<TData> where TData: PackedStructSlice {
    fn pack_to_slice(&self, output: &mut [u8]) -> Result<(), packed_struct::PackingError> {
        if output.len() != Self::packed_bytes(){
            return Err(PackingError::BufferTooSmall);
        }

        let mut i = 0;

        let n = FractalHeader::packed_bytes();
        self.header.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        let n = 1;
        output[i] = self.function.to_primitive();
        i += n;
        
        let n = TData::packed_bytes();
        self.data.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        let n = FractalFooter::packed_bytes();
        self.footer.pack_to_slice(&mut output[i..(i+n)])?;

        Ok(())
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, packed_struct::PackingError> {
        if src.len() != Self::packed_bytes() { return Err(packed_struct::PackingError::BufferSizeMismatch {actual: src.len(), expected: Self::packed_bytes() }); }

        let mut i = 0;

        let n = FractalHeader::packed_bytes();
        let header = FractalHeader::unpack_from_slice(&src[i..(i+n)])?;
        i += n;

        let n = 1;
        let function = match FractalFunction::from_primitive(src[i]) {
            Some(f) => f,
            None => {
                return Err(packed_struct::PackingError::InvalidValue);
            }
        };
        i += n;

        let n = TData::packed_bytes();
        let data = TData::unpack_from_slice(&src[i..(i+n)])?;
        i += n;

        let n = FractalFooter::packed_bytes();
        let footer = FractalFooter::unpack_from_slice(&src[i..(i+n)])?;

        let r = Self {
            header,
            function,
            data,
            footer
        };

        Ok(r)
    }

    fn packed_bytes() -> usize {
        FractalHeader::packed_bytes() + 1 + TData::packed_bytes() + FractalFooter::packed_bytes()
    }
}

impl<TData> FractalMessageChecksum for FractalAudioMessage<TData> where TData: PackedStructSlice {
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

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct DataVoid;

impl PackedStructSlice for DataVoid {
    fn pack_to_slice(&self, output: &mut [u8]) -> Result<(), PackingError> {
        Ok(())
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, PackingError> {
        Ok(DataVoid)
    }

    fn packed_bytes() -> usize {
        0
    }
}

pub struct Data<T1, T2>(T1, T2);
impl<T1, T2> PackedStructSlice for Data<T1, T2> where T1: PackedStructSlice, T2: PackedStructSlice {
    fn pack_to_slice(&self, output: &mut [u8]) -> Result<(), PackingError> {
        if output.len() != Self::packed_bytes() { return Err(PackingError::BufferSizeMismatch { expected: Self::packed_bytes(), actual: output.len() }); }

        let mut i = 0;

        let n = T1::packed_bytes();
        self.0.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        let n = T2::packed_bytes();
        self.1.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        Ok(())
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, PackingError> {
        let mut i = 0;

        let n = T1::packed_bytes();
        let t1 = T1::unpack_from_slice(&src[i..(i+n)])?;
        i += n;

        let n = T2::packed_bytes();
        let t2 = T2::unpack_from_slice(&src[i..(i+n)])?;
        i += n;

        Ok(Self(t1, t2))
    }

    fn packed_bytes() -> usize {
        T1::packed_bytes() + T2::packed_bytes()
    }
}

#[test]
fn test_generics() {
    use super::FractalU7;

    let msg = FractalAudioMessage::<FractalU7>::new(FractalModel::III, FractalFunction::GET_SCENE_NAME, FractalU7::new_all());
    let packed = msg.pack_to_vec().unwrap();
    println!("{:#X?}", packed);

    let unpacked = FractalAudioMessage::<FractalU7>::unpack_from_slice(&packed).unwrap();
    assert_eq!(msg, unpacked);
}