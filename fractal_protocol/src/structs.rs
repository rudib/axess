use packed_struct::PackedStruct;

use crate::{functions::FractalFunction, model::FractalModel, message2::SYSEX_HEADER, message2::SYSEX_START, message2::SYSEX_MANUFACTURER, message2::SYSEX_END};

/*
#[derive(PackedStruct)]
#[packed_struct(bit_numbering="msb0")]
pub struct TestPack {
    #[packed_field(bits="0..=2")]
    tiny_int: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="3..=4", ty="enum")]
    mode: SelfTestMode,
    #[packed_field(bits="7")]
    enabled: bool
}
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FractalInt {
    Int(u16),
    All
}

//pub trait FractalCmdBase {
//    fn init(model: FractalModel) -> Self;
//}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmd {
    pub sysex_message_start: u8,
    pub manufacturer: [u8; 3],
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub model: FractalModel,
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub function: FractalFunction,
    pub checksum: u8,
    pub sysex_message_stop: u8
}

impl FractalCmd {
    pub fn new(model: FractalModel, function: FractalFunction) -> Self {
        let mut cmd = FractalCmd {
            sysex_message_start: SYSEX_START,
            manufacturer: SYSEX_MANUFACTURER,
            model: model,
            function: function,
            checksum: 0,
            sysex_message_stop: SYSEX_END
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmd {
    fn get_checksum(&self) -> u8 {
        self.checksum
    }

    fn set_checksum(&mut self, checksum: u8) {
        self.checksum = checksum
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len()-2].to_vec()
    }
}

#[derive(Debug, Copy, Clone, PackedStruct, PartialEq, Eq)]
pub struct FractalCmdWithInt {
    pub sysex_message_start: u8,
    pub manufacturer: [u8; 3],
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub model: FractalModel,
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub function: FractalFunction,
    #[packed_field(element_size_bytes="2")]
    pub int: FractalInt,
    pub checksum: u8,
    pub sysex_message_stop: u8
}

impl FractalCmdWithInt {
    pub fn new(model: FractalModel, function: FractalFunction, int: FractalInt) -> Self {
        let mut cmd = FractalCmdWithInt {
            sysex_message_start: SYSEX_START,
            manufacturer: SYSEX_MANUFACTURER,
            model: model,
            function: function,
            int,
            checksum: 0,
            sysex_message_stop: SYSEX_END
        };
        cmd.prepare_checksum();
        cmd
    }
}

impl FractalMessageChecksum for FractalCmdWithInt {
    fn get_checksum(&self) -> u8 {
        self.checksum
    }

    fn set_checksum(&mut self, checksum: u8) {
        self.checksum = checksum
    }

    fn get_checksum_payload(&self) -> Vec<u8> {
        let a = self.pack();
        a[..a.len()-2].to_vec()
    }
}


pub trait FractalMessageChecksum {
    fn get_checksum(&self) -> u8;
    fn set_checksum(&mut self, checksum: u8);
    fn get_checksum_payload(&self) -> Vec<u8>;

    fn prepare_checksum(&mut self) {
        let crc = calc_checksum(&self.get_checksum_payload());
        self.set_checksum(crc);
    }

    fn valid_checksum(&self) -> bool {
        let crc_calculated = calc_checksum(&self.get_checksum_payload());
        self.get_checksum() == crc_calculated
    }
}

fn calc_checksum(sysex: &[u8]) -> u8 {
    if sysex.len() < 2 { return 0; }

    let mut sum = sysex[0];
    for b in &sysex[1..] {
        sum ^= *b;
    }
    sum & 0x7F
}

impl PackedStruct<[u8; 2]> for FractalInt {
    fn pack(&self) -> [u8; 2] {
        match self {
            FractalInt::Int(n) => [
                (n & 0x7F) as u8,
                ((n >> 7) & 0x7F) as u8
            ],
            FractalInt::All => [0x7F, 0x7F]
        }
        
    }

    fn unpack(src: &[u8; 2]) -> Result<Self, packed_struct::PackingError> {
        if src == &[0x7F, 0x7F] {
            Ok(FractalInt::All)
        } else {
            Ok(
                FractalInt::Int(
                    ((src[0] & 0x7F) as u16) | 
                    (((src[1] & 0x7F) as u16) << 7)
                )
            )
        }
    }
}

#[test]
fn test_pack_cmd_with_int() {
    let mut msg = FractalCmdWithInt {
        sysex_message_start: SYSEX_START,
        manufacturer: SYSEX_MANUFACTURER,
        model: FractalModel::III,
        function: FractalFunction::GET_PRESET_NAME,
        int: FractalInt::Int(150),
        checksum: 0,
        sysex_message_stop: SYSEX_END
    };
    msg.prepare_checksum();
    println!("{:?}", msg);
    let packed = msg.pack();
    println!("packed: {:#X?}", packed);

    let unpacked = FractalCmdWithInt::unpack(&packed).unwrap();
    assert!(unpacked.valid_checksum());
    assert_eq!(msg, unpacked);
}