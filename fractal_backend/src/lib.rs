//extern crate bus;
extern crate broadcaster;

use std::{time::Duration, thread, sync::{Mutex, Arc}};
//use bus::{BusReader, Bus};
use fractal_core::midi::{Midi, MidiPorts};
use broadcaster::BroadcastChannel;
use futures::executor::block_on;

#[derive(Debug, Clone)]
pub enum UiPayload {
    ListMidiPorts,
    DetectedMidiPorts {
        ports: MidiPorts
    },

    /// Hard shutdown
    Drop
}

#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}

impl Drop for UiApi {
    fn drop(&mut self) {
        // kill the thread
        //self.input.broadcast(UiRequest::new(UiCommand::Drop))
    }
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
                    Some(_) => { },
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