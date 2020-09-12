use super::FractalFooter;

pub trait FractalMessageChecksum {
    fn get_footer(&self) -> &FractalFooter;
    fn get_footer_mut(&mut self) -> &mut FractalFooter;
    fn get_checksum_payload(&self) -> Vec<u8>;

    fn prepare_checksum(&mut self) {
        let crc = calc_checksum(&self.get_checksum_payload());
        let mut footer = self.get_footer_mut();
        footer.checksum = crc;
    }

    fn valid_checksum(&self) -> bool {
        let crc_calculated = calc_checksum(&self.get_checksum_payload());
        self.get_footer().checksum == crc_calculated
    }
}

fn calc_checksum(sysex: &[u8]) -> u8 {
    if sysex.len() < 2 {
        return 0;
    }

    let mut sum = sysex[0];
    for b in &sysex[1..] {
        sum ^= *b;
    }
    sum & 0x7F
}