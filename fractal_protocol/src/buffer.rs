use log::trace;

use crate::{consts::SYSEX_HEADER, messages::FractalAudioMessages, consts::SYSEX_END, messages::parse_sysex_message};

#[derive(Debug)]
pub struct MessagesBuffer {
    buffer: Vec<u8>
}

impl MessagesBuffer {
    pub fn new() -> Self {
        MessagesBuffer {
            buffer: vec![]
        }
    }

    pub fn parse(&mut self, msg: &[u8]) -> Vec<FractalAudioMessages> {
        self.buffer.extend(msg);

        let mut last_sysex_end = None;
        let mut ret = vec![];
        
        // find the first sysex message in the buffer
        for i in 0..self.buffer.len() {
            if self.buffer[i..].starts_with(&SYSEX_HEADER) {
                for n in (i+SYSEX_HEADER.len())..self.buffer.len() {
                    match self.buffer.get(n) {
                        Some(f) if *f == SYSEX_END => {

                            match parse_sysex_message(&self.buffer[i..n+1]) {
                                Ok(msg) => {
                                    ret.push(msg);
                                }
                                Err(e) => {
                                    trace!("Failed to parsed SYSEX, entire buffer: {:X?}. Error message {:?}. Our msg buffer: {:#X?}", &self.buffer, e, &self.buffer[i..n+1]);
                                }
                            }

                            last_sysex_end = Some(n);
                        }
                        _ => ()
                    }
                }
            }
        }
        if let Some(last_sysex_end) = last_sysex_end {
            if self.buffer.len() > last_sysex_end {
                self.buffer = self.buffer[last_sysex_end+1..].to_vec();
            } else {
                self.buffer.clear();
            }
        }

        ret
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
        assert!(r.is_empty());
    }

    let mut r1 = None;
    for &m in &msg1 {
        let r = buffer.parse(&[m]);
        if r.first().is_some() { r1 = r.first().cloned(); }
    }
    assert!(r1.is_some());
    
    for r in random.clone() {
        let r = buffer.parse(&[r]);
        assert!(r.is_empty());
    }

    let mut r2 = None;
    for &m in &msg2 {
        let r = buffer.parse(&[m]);
        if r.first().is_some() { r2 = r.first().cloned(); }
    }
    assert!(r2.is_some());

    assert_eq!(0, buffer.buffer.len());
}