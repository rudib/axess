use super::common::*;
use super::model::*;
use crate::packed_struct::PrimitiveEnum;

#[derive(PartialEq, Debug, Clone)]
pub enum FractalMessage {
    Unknown(MidiMessage),
    //StatusDump(Vec<EffectStatus>),
    LooperState {
        record: bool,
        play: bool,
        overdub: bool,
        once: bool,
        reverse: bool,
        half_speed: bool,
    },
    CurrentPresetNumber(u32),
    PresetName(u32, String),
    SceneName(u8, String),
    CurrentPresetName(String),
    CurrentSceneNumber(u8),
    CurrentTempo(u32),
    FirmwareVersion {
        major: u8,
        minor: u8,
    },
    FrontPanelChangeDetected,
    MIDITempoBeat,
    MIDIChannel(u8),
    TunerInfo {
        note: u8,
        string_number: u8,
        tuner_data: u8,
    },
    
    //PresetBlocksFlags(Vec<BlockFlags>),
    //BlockGrid([[BlockGridBlock; 4]; 16]),
    /*
    BlockParameters {
        effect_id: u32,
        effect: Effect,
        parameter_id: u32,
        parameter: Parameter,
        value_raw: u32,
    },
    */
    TunerStatus(TunerStatus),
    MultipurposeResponse {
        function_id: u8,
        response_code: u8,
    },
}

#[derive(Debug, Clone)]
pub struct FractalMessageWrapper {
    pub model_raw: Option<u8>,
    pub model: Option<FractalModel>,
    pub message: FractalMessage
}



pub fn parse_message(msg: &[u8]) -> FractalMessageWrapper {
    let model_raw = msg.get(4).cloned();

    let model: Option<FractalModel> = model_raw
        .map(FractalModel::from_primitive)
        .unwrap_or(None);

    let function_id = msg.get(5);


    enum ModelGroup { Three, Legacy };
    let model_group = match model {
        Some(FractalModel::III) | Some(FractalModel::FM3) => ModelGroup::Three,
        _ => ModelGroup::Legacy
    };

    let content = match (model_group, function_id) {
        /*
        (Some(FractalModel::III), Some(0x13)) => {
            parse_status_dump(msg.into_iter().skip(6).collect())
        }
        */
        //(Some(FractalModel::III), Some(0x0F)) => parse_looper_state(msg.iter().nth(6).unwrap()),
        (ModelGroup::Three, Some(0x14)) => FractalMessage::CurrentTempo(decode_effect_id(
            msg.iter().nth(6).unwrap(),
            msg.iter().nth(7).unwrap(),
        )),
        (_, Some(0x14)) => FractalMessage::CurrentPresetNumber(decode_preset_number(
            *msg.iter().nth(6).unwrap(),
            *msg.iter().nth(7).unwrap(),
        )),
        (_, Some(0x21)) => FractalMessage::FrontPanelChangeDetected,
        //(_, Some(0x01)) => decode_block_parameters(msg),
        (_, Some(0x08)) => FractalMessage::FirmwareVersion {
            major: *msg.iter().nth(6).unwrap() as u8,
            minor: *msg.iter().nth(7).unwrap() as u8,
        },
        (ModelGroup::Three, Some(0x0D)) => FractalMessage::PresetName(
            decode_effect_id(msg.iter().nth(6).unwrap(), msg.iter().nth(7).unwrap()),
            decode_preset_name(&msg[8..]),
        ),
        (_, Some(0x0F)) => {
            FractalMessage::CurrentPresetName(decode_preset_name(&msg[6..]))
        }
        (_, Some(0x10)) => FractalMessage::MIDITempoBeat,
        (_, Some(0x11)) => FractalMessage::TunerStatus(if *msg.iter().nth(6).unwrap() == 0 as u8 {
            TunerStatus::Off
        } else {
            TunerStatus::On
        }),
        (_, Some(0x17)) => FractalMessage::MIDIChannel(1 + *msg.iter().nth(6).unwrap() as u8),
        (_, Some(0x0D)) => FractalMessage::TunerInfo {
            note: *msg.iter().nth(6).unwrap() as u8,
            string_number: *msg.iter().nth(7).unwrap() as u8,
            tuner_data: *msg.iter().nth(8).unwrap() as u8,
        },
        (ModelGroup::Three, Some(0x0E)) => FractalMessage::SceneName(
            *msg.iter().nth(6).unwrap(),
            decode_preset_name(&msg[7..]),
        ),
        /*
        (_, Some(0x0E)) => FractalMessage::PresetBlocksFlags(decode_preset_blocks_flags(
            msg.into_iter().skip(6).collect(),
        )),
        */
        /*
        (_, Some(0x20)) => {
            FractalMessage::BlockGrid(decode_block_grid(msg.into_iter().skip(6).collect()))
        }
        */
        (_, Some(0x29)) => {
            FractalMessage::CurrentSceneNumber(1 + *msg.iter().nth(6).unwrap() as u8)
        }
        (ModelGroup::Three, Some(0x0C)) => {
            FractalMessage::CurrentSceneNumber(*msg.iter().nth(6).unwrap() as u8)
        }
        (_, Some(0x64)) => FractalMessage::MultipurposeResponse {
            function_id: *msg.iter().nth(6).unwrap() as u8,
            response_code: *msg.iter().nth(7).unwrap() as u8,
        },
        _ => FractalMessage::Unknown(msg.to_vec()),
    };

    FractalMessageWrapper {
        model_raw: model_raw,
        model: model,
        message: content
    }
}

