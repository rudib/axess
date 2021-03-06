use std::fmt::Display;

use crate::messages::firmware_version::FirmwareVersion;

#[derive(Clone, Debug)]
pub struct FractalDevice {
    pub model: FractalModel,
    pub firmware: FirmwareVersion
}

impl Display for FractalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, firmware {}.{:0>2}", self.model, self.firmware.major, self.firmware.minor)
    }
}


#[derive(PrimitiveEnum, Debug, Clone, Copy, Eq, PartialEq)]
pub enum FractalModel {
    Standard = 0x00,
    Ultra = 0x01,
    MFC101 = 0x02,
    II = 0x03,
    MFC101MK3 = 0x04,
    FX8 = 0x05,
    IIXL = 0x06,
    IIXLPlus = 0x07,
    AX8 = 0x08,
    FX8MK2 = 0x0A,
    III = 0x10,
    FM3 = 0x11
}

impl Display for FractalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractalModel::Standard => f.write_str("Axe-FX Standard"),
            FractalModel::Ultra => f.write_str("Axe-FX Ultra"),
            FractalModel::MFC101 => f.write_str("MFC101"),
            FractalModel::II => f.write_str("Axe-FX II"),
            FractalModel::MFC101MK3 => f.write_str("MFC101 MK3"),
            FractalModel::FX8 => f.write_str("FX8"),
            FractalModel::IIXL => f.write_str("Axe-FX II XL"),
            FractalModel::IIXLPlus => f.write_str("Axe-FX II XL+"),
            FractalModel::AX8 => f.write_str("AX8"),
            FractalModel::FX8MK2 => f.write_str("FX8 MK2"),
            FractalModel::III => f.write_str("Axe-FX III"),
            FractalModel::FM3 => f.write_str("FM3")
        }
    }
}

impl FractalModel {
    pub fn number_of_presets(&self) -> Option<u16> {
        match self {
            FractalModel::III | FractalModel::FM3 => Some(512),
            _ => None
        }
    }

    pub fn number_of_scenes(&self) -> Option<u8> {
        match self {
            FractalModel::III | FractalModel::FM3 => Some(8),
            _ => None
        }
    }
}