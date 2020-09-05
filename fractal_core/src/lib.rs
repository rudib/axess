extern crate midir;
extern crate log;
#[macro_use] extern crate quick_error;

pub mod midi;

quick_error! {
    #[derive(Debug)]
    pub enum FractalCoreError {
        Unknown {}
        MidirInit(err: midir::InitError) { from() }
        MidirConnectInput(err: midir::ConnectError<midir::MidiInput>) { from() }
        MidirConnectOutput(err: midir::ConnectError<midir::MidiOutput>) { from() }
    }
}
