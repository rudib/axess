use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct FractalDevice {
    pub model: FractalModel,
    pub firmware: (u8, u8)
}

impl Display for FractalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, firmware {}.{}", self.model, self.firmware.0, self.firmware.1)
    }
}


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FractalModel {
    Standard,
    Ultra,
    MFC101,
    II,
    MFC101MK3,
    FX8,
    IIXL,
    IIXLPlus,
    AX8,
    FX8MK2,
    III,
    FM3
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
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(FractalModel::Standard),
            0x01 => Some(FractalModel::Ultra),
            0x02 => Some(FractalModel::MFC101),
            0x03 => Some(FractalModel::II),
            0x04 => Some(FractalModel::MFC101MK3),
            0x05 => Some(FractalModel::FX8),
            0x06 => Some(FractalModel::IIXL),
            0x07 => Some(FractalModel::IIXLPlus),
            0x08 => Some(FractalModel::AX8),
            0x0A => Some(FractalModel::FX8MK2),
            0x10 => Some(FractalModel::III),
            0x11 => Some(FractalModel::FM3),
            _ => None
        }
    }
}

pub fn model_code(model: FractalModel) -> u8 {
    match model {
        FractalModel::Standard => 0x00,
        FractalModel::Ultra => 0x01,
        FractalModel::MFC101 => 0x02,
        FractalModel::II => 0x03,
        FractalModel::MFC101MK3 => 0x04,
        FractalModel::FX8 => 0x05,
        FractalModel::IIXL => 0x06,
        FractalModel::IIXLPlus => 0x07,
        FractalModel::AX8 => 0x08,
        FractalModel::FX8MK2 => 0x0A,
        FractalModel::III => 0x10,
        FractalModel::FM3 => 0x11
    }
}
