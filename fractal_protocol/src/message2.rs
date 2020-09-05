use crate::{
    common::calc_checksum,
    message::{parse_message, FractalMessageWrapper},
};
use log::error;

const SYSEX_MANUFACTURER_BYTE1: u8 = 0x00;
const SYSEX_MANUFACTURER_BYTE2: u8 = 0x01;
const SYSEX_MANUFACTURER_BYTE3: u8 = 0x74;

pub fn validate_and_decode_message(msg: &[u8]) -> Option<FractalMessageWrapper> {
    let mut valid_msg = false;

    if msg.len() > 5
        && msg[0] == 0xF0
        && msg[1] == SYSEX_MANUFACTURER_BYTE1
        && msg[2] == SYSEX_MANUFACTURER_BYTE2
        && msg[3] == SYSEX_MANUFACTURER_BYTE3
        && msg[msg.len() - 1] == 0xF7
    {
        let crc_byte = msg[msg.len() - 2];
        let calc = calc_checksum(&msg[..msg.len() - 2]);
                
        if calc == crc_byte {
            valid_msg = true;
        } else {
            error!("CRC mismatch: CRC byte {:#X}, calculated {:#X}", crc_byte, calc);
        }
    }

    // decode the sysex message
    if valid_msg {
        let fractal_msg = parse_message(&msg);
        return Some(fractal_msg);
    }

    None
}
