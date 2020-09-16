pub mod buffer;
pub mod model;
pub mod functions;
pub mod effect;

pub mod structs;

pub mod messages;

extern crate packed_struct;
#[macro_use]
extern crate packed_struct_codegen;

#[macro_use] extern crate quick_error;

#[macro_use] extern crate derive_more;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum FractalProtocolError {
        CrcMismatch {}
        UnknownMessage {}
        MessageConversionError {}
        UnknownValue {param: String, value: String}
    }
}
