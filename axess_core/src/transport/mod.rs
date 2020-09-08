pub mod midi;
pub mod serial;


use crate::FractalCoreError;
use crossbeam_channel::{Sender, Receiver};

pub type TransportMessage = Vec<u8>;

#[derive(Debug, Clone)]
pub struct TransportEndpoint {
    pub id: String,
    pub name: String
}

pub trait Transport {
    type TConnection: TransportConnection;

    fn id() -> String;
    fn detect_endpoints(&self) -> Result<Vec<TransportEndpoint>, FractalCoreError>;
    fn connect(&self, endpoint: &TransportEndpoint) -> Result<Self::TConnection, FractalCoreError>;
}

pub trait TransportConnection {
    fn get_receiver(&self) -> &Receiver<TransportMessage>;
    fn get_sender(&self) -> &Sender<TransportMessage>;
}
