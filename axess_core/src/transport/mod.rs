pub mod midi;
pub mod serial;


use crate::{FractalResultVoid, FractalCoreError};
use crossbeam_channel::{Receiver};
use fractal_protocol::structs::FractalAudioMessagePacker;

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

    fn write_msg(&mut self, msg: &mut dyn FractalAudioMessagePacker) -> FractalResultVoid {
        let packed = msg.pack()?;
        self.write(&packed)
    }
}

pub fn write_struct<T: TransportConnection, P: FractalAudioMessagePacker>(connection: &mut T, s: &mut P) -> FractalResultVoid {
    let msg = s.pack()?;
    connection.write(&msg)
}

pub fn write_struct_dyn<P: FractalAudioMessagePacker>(connection: &mut dyn TransportConnection, s: &mut P) -> FractalResultVoid {
    let msg = s.pack()?;
    connection.write(&msg)
}