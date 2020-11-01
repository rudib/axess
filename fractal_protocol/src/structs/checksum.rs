pub fn calc_checksum(sysex: &[u8]) -> u8 {
    if sysex.len() < 2 {
        return 0;
    }

    let mut sum = sysex[0];
    for b in &sysex[1..] {
        sum ^= *b;
    }
    sum & 0x7F
}