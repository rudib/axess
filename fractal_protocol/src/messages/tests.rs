use super::parse_sysex_message;

#[test]
fn should_all_parse() {
    let messages = [
        vec![0xF0, 0x0, 0x1, 0x74, 0x11, 0xC, 0x1, 0x19, 0xF7],
        vec![
            0xF0, 0x0, 0x1, 0x74, 0x11, 0xD, 0x49, 0x2, 0x45, 0x72, 0x75, 0x70, 0x74, 0x69, 0x6F,
            0x6E, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x0, 0x5C, 0xF7,
        ],
    ];

    for (i, msg) in messages.iter().enumerate() {
        let msg = parse_sysex_message(&msg).expect(&format!("message {} failed to parse", i));
        println!("message {}: {:?}", i, msg);
    }
}
