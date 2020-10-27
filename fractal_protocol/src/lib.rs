#![allow(unused_imports)]

pub mod buffer;
pub mod model;
pub mod functions;
pub mod effect;

pub mod structs;

pub mod messages;
pub mod consts;

extern crate packed_struct;
#[macro_use]
extern crate packed_struct_codegen;

#[macro_use] extern crate quick_error;

#[macro_use] extern crate derive_more;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum FractalProtocolError {
        ConstantMismatch { constant: String }
        CrcMismatch { message: u8, calculated: u8 }
        UnknownMessage {}
        MessageConversionError {}
        UnknownValue {param: String, value: String}
        PackingError(err: packed_struct::PackingError) { from() }
    }
}
