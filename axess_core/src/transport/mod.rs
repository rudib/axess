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
