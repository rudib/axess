pub extern crate midir;
pub extern crate bus;

#[macro_use] extern crate quick_error;

pub mod midi;

quick_error! {
    #[derive(Debug)]
    pub enum FractalCoreError {
        Unknown {}
        MidirInit(err: midir::InitError) { from() }
    }
}
