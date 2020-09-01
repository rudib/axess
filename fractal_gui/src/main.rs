extern crate fractal_protocol;
extern crate fractal_core;

use fractal_core::midi::*;

fn main() {
    let midi = Midi::new();
    let midi_ports = midi.detect_midi_ports().unwrap();
    println!("all midi ports: {:?}", midi_ports);
    let fractals = midi_ports.detect_fractal_devices();
    println!("fractals: {:?}", fractals);

    
}