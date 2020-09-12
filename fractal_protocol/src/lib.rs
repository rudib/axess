pub mod buffer;
pub mod common;
pub mod message;
pub mod message2;
pub mod model;
pub mod functions;
pub mod effect;

pub mod structs;
pub mod commands;

pub mod messages;

extern crate packed_struct;
#[macro_use]
extern crate packed_struct_codegen;

#[macro_use] extern crate quick_error;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum FractalProtocolError {
        CrcMismatch {}
        UnknownMessage {}
    }
}