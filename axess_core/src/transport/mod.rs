pub mod midi;
pub mod serial;


use crate::{FractalResultVoid, FractalCoreError};
use crossbeam_channel::{Receiver};

pub type TransportMessage = Vec<u8>;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub transport_id: String,
    pub transport_endpoint: TransportEndpoint
}

#[derive(Debug, Clone)]
pub struct TransportEndpoint {
    pub id: String,
    pub name: String
}

pub trait Transport {
    fn id(&self) -> String;
    fn detect_endpoints(&self) -> Result<Vec<TransportEndpoint>, FractalCoreError>;
    fn connect(&self, endpoint: &TransportEndpoint) -> Result<Box<dyn TransportConnection>, FractalCoreError>;
}

pub trait TransportConnection {
    fn get_receiver(&self) -> &Receiver<TransportMessage>;
    fn write(&mut self, buf: &[u8]) -> FractalResultVoid;
}

pub fn write_struct<T: TransportConnection, P: packed_struct::PackedStructSlice>(connection: &mut T, s: &P) -> FractalResultVoid {
    let msg = s.pack_to_vec()?;
    connection.write(&msg)
}

pub fn write_struct_dyn<P: packed_struct::PackedStructSlice>(connection: &mut dyn TransportConnection, s: &P) -> FractalResultVoid {
    let msg = s.pack_to_vec()?;
    connection.write(&msg)
}