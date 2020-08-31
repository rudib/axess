pub type MidiMessage = Vec<u8>;

pub fn decode_effect_id(a: &u8, b: &u8) -> u32 {
    let a: u32 = (*a).into();
    let b: u32 = (*b).into();
    (a & 0x7F) | ((b & 0x7F) << 7)
}

pub fn decode_preset_number(lsb: u8, rsb: u8) -> u32 {
    (((lsb as u32) & 0x7F) << 7) | (rsb as u32)
}

pub fn decode_preset_name(msg: &[u8]) -> String {
    msg.iter()
        .take(32)
        .filter(|x| *x > &0)
        .map(|x| *x as u8 as char)
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TunerStatus {
    On,
    Off,
}


pub fn checksum(msg: MidiMessage) -> u8 {
    let xord = msg
        .iter()
        .take(msg.len() - 1)
        .fold(None, |acc: Option<u8>, x| match acc {
            Some(y) => Some(y ^ x),
            None => Some(*x),
        })
        .unwrap();
    0x7F & xord
}

pub fn calc_checksum(sysex: &[u8]) -> u8 {
    if sysex.len() < 2 { return 0; }

    let mut sum = sysex[0];
    for b in &sysex[1..] {
        sum ^= *b;
    }
    sum & 0x7F
}

pub fn with_checksum(msg: MidiMessage) -> MidiMessage {
    let term = msg.iter().last().unwrap();
    let msg_checksum = checksum(msg.clone());
    let msg_without_term: MidiMessage = msg
        .clone()
        .into_iter()
        .take(msg.len() - 1)
        .collect::<Vec<u8>>();
    [msg_without_term, vec![msg_checksum, *term]].concat()
}

pub fn wrap_msg(msg: MidiMessage) -> MidiMessage {
    let header = vec![0xF0, 0x00, 0x01, 0x74];
    with_checksum([header, msg, vec![0xF7]].concat())
}