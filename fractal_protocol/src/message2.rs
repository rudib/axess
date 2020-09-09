use crate::{
    common::calc_checksum,
    message::{parse_message, FractalMessageWrapper},
};
use log::error;

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_MANUFACTURER_BYTE1: u8 = 0x00;
pub const SYSEX_MANUFACTURER_BYTE2: u8 = 0x01;
pub const SYSEX_MANUFACTURER_BYTE3: u8 = 0x74;
pub const SYSEX_END: u8 = 0xF7;

pub fn validate_and_decode_message(msg: &[u8]) -> Option<FractalMessageWrapper> {
    let mut valid_msg = false;

    if msg.len() > 5
        && msg[0] == SYSEX_START
        && msg[1] == SYSEX_MANUFACTURER_BYTE1
        && msg[2] == SYSEX_MANUFACTURER_BYTE2
        && msg[3] == SYSEX_MANUFACTURER_BYTE3
        && msg[msg.len() - 1] == SYSEX_END
    {
        let crc_byte = msg[msg.len() - 2];
        let calc = calc_checksum(&msg[..msg.len() - 2]);
                
        if calc == crc_byte {
            valid_msg = true;
        } else {
            error!("CRC mismatch: CRC byte {:#X}, calculated {:#X}. Entire message: {:X?}", crc_byte, calc, msg);
        }
    }

    // decode the sysex message
    if valid_msg {
        let fractal_msg = parse_message(&msg);
        return Some(fractal_msg);
    }

    None
}
