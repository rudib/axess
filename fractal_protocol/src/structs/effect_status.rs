use std::convert::TryInto;

use packed_struct::{types::{Integer, ReservedZero}, PackedStruct};
use packed_struct::prelude::packed_bits;

use crate::{functions::FractalFunction, messages::parse_sysex_message, messages::effects::Blocks};

use super::{FractalU14, FractalHeader, FractalFooter, FractalMessageChecksum, FractalAudioMessage};
use packed_struct::{PrimitiveEnum, PackedStructSlice};

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(bit_numbering = "msb0", size_bytes="3")]
pub struct EffectStatus {
    #[packed_field(element_size_bytes = "2")]
    pub effect_id: FractalU14,
    pub _padding: ReservedZero<packed_bits::Bits1>,
    pub supported_channels: Integer<u8, packed_bits::Bits3>,
    pub channel: Integer<u8, packed_bits::Bits3>,
    pub is_bypassed: bool
}

#[test]
fn test_effect_status_dump() {
    use crate::effect::EffectId;

    let msg = vec![
        0xf0, 0x00, 0x01, 0x74, 0x10, 0x13, 0x3a, 0x00, 0x40, 0x3e, 0x00, 0x40, 0x2e, 0x00, 0x40,
        0x46, 0x00, 0x40, 0x7a, 0x00, 0x40, 0x3a, 0x01, 0x10, 0x36, 0x01, 0x10, 0x76, 0x00, 0x42,
        0x77, 0x00, 0x41, 0x25, 0x00, 0x40, 0x7e, 0x00, 0x40, 0x2a, 0x00, 0x40, 0x6e, 0x00, 0x41,
        0x32, 0x01, 0x40, 0x42, 0x00, 0x40, 0x43, 0x00, 0x40, 0x6a, 0x00, 0x41, 0x66, 0x00, 0x40,
        0x26, 0x01, 0x10, 0x49, 0x01, 0x10, 0x48, 0x01, 0x10, 0x09, 0xf7,
    ];
    
    let es_packed = [0x3a, 0x00, 0x40];
    let es = EffectStatus::unpack(&es_packed).unwrap();    
    println!("es: {:?}", es);
    println!("{}", es);
    //let effect_id: Option<EffectId> = es.effect_id.into();
    let effect_id: u16 = es.effect_id.into();
    assert_eq!(Some(EffectId::ID_AMP1), EffectId::from_primitive(effect_id as u8));
    assert_eq!(&es_packed, &es.pack());

    let all_effects = FractalAudioMessage::<Vec<EffectStatus>>::unpack_from_slice(&msg).unwrap();
    println!("{:?}", all_effects);
    
    //assert!(all_effects.valid_checksum());
}


#[test]
fn test_parse_axe3() {
    extern crate log4rs;
    use log4rs::{append::{console::ConsoleAppender, file::FileAppender, console::Target}, config::Config, config::Appender, config::Root};
    use log::{trace, LevelFilter};

        // init logging
        {
            let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
            let file = FileAppender::builder().build("axess-test.log").unwrap();
            let config = Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(stdout)))
                .appender(Appender::builder().build("file", Box::new(file)))
                .build(Root::builder()
                    .appender("stdout")
                    .appender("file")
                    .build(LevelFilter::Trace)
                ).unwrap();
            log4rs::init_config(config).unwrap();
        }

    let msg = [
        0xF0,
        0x0,
        0x1,
        0x74,
        0x10,
        0xE,
        0x1B,
        0x0,
        0x0,
        0x0,
        0x3F,
        0x5D,
        0x4D,
        0x2E,
        0x3F,
        0x0,
        0x0,
        0x3F,
        0x0,
        0x0,
        0x3F,
        0x3E,
        0x0,
        0x0,
        0x0,
        0x0,
        0x4D,
        0x26,
        0x3F,
        0x73,
        0x3E,
        0x36,
        0xF7,
    ];
    
    let raw = FractalAudioMessage::<Vec<EffectStatus>>::unpack_from_slice(&msg).unwrap();
    println!("raw: {:?}", raw);


    let parsed: Blocks = parse_sysex_message(&msg).unwrap().try_into().unwrap();
    println!("blocks: {:?}", parsed);
}