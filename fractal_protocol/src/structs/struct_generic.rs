use packed_struct::{PackedStructSlice, PackingError, PrimitiveEnum, types::bits::ByteArray};

use crate::{functions::FractalFunction, model::FractalModel, messages::preset::PresetHelper};
use super::{FractalHeader, FractalFooter, FractalMessageChecksum, FractalU14};
#[derive(Debug, Clone, PartialEq)]
pub struct FractalAudioMessage<TData> {
    pub header: FractalHeader,
    pub function: FractalFunction,

    pub data: TData,

    pub footer: FractalFooter
}

pub trait FractalAudioMessageFunction {
    fn get_function(&self) -> FractalFunction;
}

impl<TData> FractalAudioMessageFunction for FractalAudioMessage<TData> {
    fn get_function(&self) -> FractalFunction {
        self.function
    }
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
        if output.len() != Self::packed_bytes_size(None)? {
            return Err(PackingError::BufferTooSmall);
        }

        let mut i = 0;

        let n = FractalHeader::packed_bytes_size(None)?;
        self.header.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        let n = 1;
        output[i] = self.function.to_primitive();
        i += n;
        
        let n = TData::packed_bytes_size(Some(&self.data))?;
        self.data.pack_to_slice(&mut output[i..(i+n)])?;
        i += n;

        let n = FractalFooter::packed_bytes_size(None)?;
        self.footer.pack_to_slice(&mut output[i..(i+n)])?;

        Ok(())
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, packed_struct::PackingError> {
        //let expected = Self::packed_bytes_size(None)?;
        //if src.len() != expected { return Err(packed_struct::PackingError::BufferSizeMismatch {actual: src.len(), expected: Self::packed_bytes() }); }


        let n_footer = FractalFooter::packed_bytes_size(None)?;

        let mut i = 0;

        let n = FractalHeader::packed_bytes_size(None)?;
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

        //let n = TData::packed_bytes();
        let data = TData::unpack_from_slice(&src[i..(src.len()-n_footer)])?;
                
        let footer = FractalFooter::unpack_from_slice(&src[(src.len() - n_footer)..])?;

        let r = Self {
            header,
            function,
            data,
            footer
        };

        Ok(r)
    }

    fn packed_bytes_size(opt_self: Option<&Self>) -> Result<usize, PackingError> {
        Ok(
            FractalHeader::packed_bytes_size(None)? + 1 + TData::packed_bytes_size(opt_self.map(|s| &s.data))? + FractalFooter::packed_bytes_size(None)?
        )
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

#[derive(Debug)]
pub struct DataBytes<TBytes> where TBytes: packed_struct::types::bits::NumberOfBytes {
    pub bytes: TBytes::AsBytes
}
impl<TBytes> PackedStructSlice for DataBytes<TBytes> where TBytes: packed_struct::types::bits::NumberOfBytes {
    fn pack_to_slice(&self, output: &mut [u8]) -> Result<(), PackingError> {
        output.copy_from_slice(&self.bytes.as_bytes_slice());
        Ok(())
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, PackingError> {
        let mut bytes = TBytes::AsBytes::new(0);
        let slice = bytes.as_mut_bytes_slice();
        if slice.len() != src.len() { return Err(PackingError::BufferSizeMismatch { expected: slice.len(), actual: src.len() }); }
        slice.copy_from_slice(src);

        Ok(DataBytes {
            bytes
        })
    }

    fn packed_bytes_size(_opt_self: Option<&Self>) -> Result<usize, PackingError> {
        Ok(TBytes::number_of_bytes() as usize)
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

/*
#[test]
fn test_numbers() {
    let bytes = [240, 0, 1, 116, 3, 20, 1, 107, 120, 247];
    let decoded = FractalAudioMessage::<FractalU14>::unpack_from_slice(&bytes).unwrap();
    let p: u16 = decoded.data.into();
    assert_eq!(235, p);

    let bytes = [240, 0, 1, 116, 3, 20, 1, 108, 121, 247];
    let decoded = FractalAudioMessage::<FractalU14>::unpack_from_slice(&bytes).unwrap();
    let p: u16 = decoded.data.into();
    assert_eq!(236, p);

    let packed = decoded.pack_to_vec().unwrap();
    assert_eq!(&bytes, packed.as_slice());
}
*/