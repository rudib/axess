use crate::structs::{FractalU7, FractalAudioMessage, FractalU14, FractalString32, Data};
use crate::FractalProtocolError;

pub enum FractalAudioMessages {
    // 0x08
    FirmwareVersion(FractalAudioMessage<Data<FractalU7, FractalU7>>),
    // 0x64
    MultipurposeResponse(FractalAudioMessage<FractalU7>),
    // 0x0D
    PresetName(FractalAudioMessage<Data<FractalU14, FractalString32>>),
    // 0x0E
    SceneName(FractalAudioMessage<Data<FractalU7, FractalString32>>)
}

pub fn parse_sysex_message(msg: &[u8]) -> Result<FractalAudioMessages, FractalProtocolError> {
    todo!();
}