extern crate midir;
extern crate log;
#[macro_use] extern crate quick_error;

pub mod midi;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum FractalCoreError {
        Timeout {}
        Unknown {}
        MissingValue(val: String) { }
        MidirInit(err: midir::InitError) { from() }
        MidirConnectError { }
        MidirSendError(err: midir::SendError) { from() }
    }
}

impl From<midir::ConnectError<midir::MidiInput>> for FractalCoreError {
    fn from(_: midir::ConnectError<midir::MidiInput>) -> Self {
        FractalCoreError::MidirConnectError
    }
}

impl From<midir::ConnectError<midir::MidiOutput>> for FractalCoreError {
    fn from(_: midir::ConnectError<midir::MidiOutput>) -> Self {
        FractalCoreError::MidirConnectError
    }
}
