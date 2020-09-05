extern crate broadcaster;

use std::{thread};
use fractal_core::midi::{Midi, MidiPorts};
use broadcaster::BroadcastChannel;
use futures::executor::block_on;

#[derive(Debug, Clone)]
pub enum UiPayload {
    Connection(PayloadConnection),
    
    /// Internal
    Ping,
    
    /// Hard shutdown
    Drop
}

#[derive(Debug, Clone)]
pub enum PayloadConnection {
    ListMidiPorts,
    DetectedMidiPorts {
        ports: MidiPorts
    },
    ConnectToMidiPorts {
        input_port: String,
        output_port: String
    },

    TryToAutoConnect,
    AutoConnectResult(bool),

    
    // Events
    Connected,
    Disconnected    
}


#[derive(Clone)]
pub struct UiApi {
    pub channel: BroadcastChannel<UiPayload>
}
/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {
    channel: BroadcastChannel<UiPayload>
}

impl UiBackend {    
    pub fn spawn() -> UiApi {
        let mut chan = BroadcastChannel::new();

        let api = UiApi {
            channel: chan.clone()
        };

        let mut backend = UiBackend {
            channel: chan
        };

        thread::Builder::new().name("Backend".into()).spawn(move || {
            loop {
                match block_on(backend.channel.recv()) {  

                    Some(UiPayload::Connection(c)) => {
                        backend.connection(c);
                    },
                    Some(_) => {

                    },
                    None => {
                        println!("end of stream!");
                        break;
                    }
                }
            }

            println!("shutting down");
        }).unwrap();

        api
    }

    fn send(&self, msg: UiPayload) {
        block_on(self.channel.send(&msg)).unwrap();
    }

    fn connection(&self, msg: PayloadConnection) {
        match msg {
            PayloadConnection::ListMidiPorts => {
                let midi = Midi::new();
                if let Ok(midi_ports) = midi.detect_midi_ports() {
                    self.send(UiPayload::Connection(PayloadConnection::DetectedMidiPorts {
                        ports: midi_ports
                    }));
                }
            }
            //PayloadConnection::DetectedMidiPorts { ports } => {}
            PayloadConnection::ConnectToMidiPorts { input_port, output_port } => {}
            PayloadConnection::TryToAutoConnect => {}
            //PayloadConnection::AutoConnectResult(_) => {}
            //PayloadConnection::Connected => {}
            //PayloadConnection::Disconnected => {}
            _ => {}
        }
    }
}