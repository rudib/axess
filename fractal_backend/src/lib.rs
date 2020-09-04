extern crate broadcaster;

use std::{thread};
use fractal_core::midi::{Midi, MidiPorts};
use broadcaster::BroadcastChannel;
use futures::executor::block_on;

#[derive(Debug, Clone)]
pub enum UiPayload {
    ListMidiPorts,
    DetectedMidiPorts {
        ports: MidiPorts
    },
    ConnectToMidiPorts {
        input_port: String,
        output_port: String
    },

    /// Internal
    Ping,
    
    /// Hard shutdown
    Drop
}

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {

}

impl UiBackend {
    fn new() -> Self {
        UiBackend {

        }
    }
    pub fn spawn() -> UiApi {
        let mut chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        thread::spawn(move || {
            loop {
                match block_on(chan.recv()) {                    
                    Some(UiPayload::ListMidiPorts) => {
                        let midi = Midi::new();
                        if let Ok(midi_ports) = midi.detect_midi_ports() {
                            block_on(chan.send(&UiPayload::DetectedMidiPorts {
                                ports: midi_ports
                            })).unwrap();
                        }
                    },
                    Some(UiPayload::ConnectToMidiPorts { input_port, output_port }) => {
                        println!("try to connect to {}, {}", input_port, output_port);
                    },
                    Some(UiPayload::Drop) => { 
                        break;
                    },
                    Some(_) => {

                    }
                    None => {
                        println!("end of stream!");
                        break;
                    }
                }
            }

            println!("shutting down");
        });

        api
    }
}