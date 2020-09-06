quick_error! {
    #[derive(Debug, Clone)]
    pub enum FractalCoreError {
        Timeout {}
        Unknown {}
        MissingValue(val: String) { }
        MidirInit(err: midir::InitError) { from() }
        MidirConnectError { }
        MidirSendError(err: midir::SendError) { from() }

        BroadcastError(val: String) { }
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

impl From<futures::channel::mpsc::SendError> for FractalCoreError {
    fn from(_: futures::channel::mpsc::SendError) -> Self {
        FractalCoreError::BroadcastError("channel".into())
    }
}

pub type FractalResult<T> = Result<T, FractalCoreError>;
pub type FractalResultVoid = FractalResult<()>;