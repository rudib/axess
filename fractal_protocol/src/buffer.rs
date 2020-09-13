use log::trace;

use crate::{message::FractalMessageWrapper, message2::SYSEX_HEADER, message2::SYSEX_END, message2::validate_and_decode_message, messages::parse_sysex_message, messages::FractalAudioMessages};

pub struct MessagesBuffer {
    buffer: Vec<u8>
}

impl MessagesBuffer {
    pub fn new() -> Self {
        MessagesBuffer {
            buffer: vec![]
        }
    }

    // todo: parse ALL messages in the buffer
    pub fn parse(&mut self, msg: &[u8]) -> Option<FractalAudioMessages> {
        self.buffer.extend(msg);

        // find the first sysex message in the buffer
        for i in 0..self.buffer.len() {
            if self.buffer[i..].starts_with(&SYSEX_HEADER) {
                for n in (i+SYSEX_HEADER.len())..self.buffer.len() {
                    match self.buffer.get(n) {
                        Some(f) if *f == SYSEX_END => {

                            match parse_sysex_message(&self.buffer[i..n+1]) {
                                Ok(msg) => {
                                    self.buffer = self.buffer[i+(n-i)+1..].to_vec();
                                    return Some(msg);
                                }
                                Err(e) => {
                                    trace!("Failed to parsed SYSEX, entire buffer: {:X?}. Error message {:?}", &self.buffer, e);
                                }
                            }
                        }
                        _ => ()
                    }
                }
            }
        }

        None
    }
}


#[test]
fn test_messages_with_random_stuff() {
    let msg1 = [0xF0, 0x0, 0x1, 0x74, 0x11, 0xD, 0x7F, 0x7F, 0x19, 0xF7];
    let msg2 = [0xF0, 0x0, 0x1, 0x74, 0x11, 0xE, 0x7F, 0x65, 0xF7];

    let random = 100..132;

    let mut buffer = MessagesBuffer::new();
    for r in random.clone() {
        let r = buffer.parse(&[r]);
        assert!(r.is_none());
    }

    let mut r1 = None;
    for &m in &msg1 {
        let r = buffer.parse(&[m]);
        if r.is_some() { r1 = r; }
    }
    assert!(r1.is_some());
    
    for r in random.clone() {
        let r = buffer.parse(&[r]);
        assert!(r.is_none());
    }

    let mut r2 = None;
    for &m in &msg2 {
        let r = buffer.parse(&[m]);
        if r.is_some() { r2 = r; }
    }
    assert!(r2.is_some());

    assert_eq!(0, buffer.buffer.len());
}