pub extern crate midir;

#[macro_use] extern crate quick_error;

pub mod midi;

quick_error! {
    #[derive(Debug)]
    pub enum FractalCoreError {
        Unknown {}
        MidirInit(err: midir::InitError) { from() }
    }
}
