use crate::model::{model_code, FractalModel};

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
        .take_while(|x| *x > &0)
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

pub fn get_current_preset_name(model: FractalModel) -> MidiMessage {
    // todo: test
    if model == FractalModel::III || model == FractalModel::FM3 {
        wrap_msg(vec![model_code(model), 0x0D, 0x7F, 0x7F])
    } else {
        wrap_msg(vec![model_code(model), 0x0F])
    }
}

pub fn get_firmware_version(model_code: u8) -> MidiMessage {
    wrap_msg(vec![model_code, 0x08])
}

pub fn disconnect_from_controller(model: FractalModel) -> MidiMessage {
    wrap_msg(vec![model_code(model), 0x42])
}

pub fn get_current_scene_name(model: FractalModel) -> MidiMessage {
    get_scene_name(model, 0x7F)
}

pub fn get_scene_name(model: FractalModel, scene: u8) -> MidiMessage {
    wrap_msg(vec![model_code(model), 0x0E, scene])
}

pub fn set_preset_number(model: FractalModel, n: u32) -> MidiMessage {
    let (a, b) = encode_preset_number(n);
    wrap_msg(vec![model_code(model), 0x3C, a, b])
}

fn encode_preset_number(n: u32) -> (u8, u8) {
    ((n >> 7) as u8, (n & 0x7F) as u8)
}

pub fn set_scene_number(model: FractalModel, scene_number: u8) -> MidiMessage {
    let command = if model == FractalModel::III || model == FractalModel::FM3 {
        0x0C
    } else {
        0x29
    };
    wrap_msg(vec![model_code(model), command, scene_number])
}


#[test]
fn test_set_preset_number() {
    assert_eq!(
        vec![
            0xF0,
            0x00,
            0x01,
            0x74,
            model_code(FractalModel::II),
            0x3C,
            0,
            127,
            69,
            0xF7
        ],
        set_preset_number(FractalModel::II, 127)
    );
    assert_eq!(
        vec![
            0xF0,
            0x00,
            0x01,
            0x74,
            model_code(FractalModel::II),
            0x3C,
            1,
            0,
            59,
            0xF7
        ],
        set_preset_number(FractalModel::II, 128)
    );
}