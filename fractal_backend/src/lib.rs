extern crate bus;

use std::{time::Duration, thread, sync::{Mutex, Arc}};
use bus::{BusReader, Bus};
use fractal_core::midi::{Midi, MidiPorts};

#[derive(Debug, Clone)]
pub struct UiRequest {
    pub command: UiCommand
}

impl UiRequest {
    pub fn new(command: UiCommand) -> Self {
        UiRequest { command: command }
    }
}

#[derive(Debug, Clone)]
pub enum UiCommand {
    ListMidiPorts,
    
    /// Hard shutdown
    Drop
}


#[derive(Debug, Clone)]
pub struct UiResponse {
    pub request: UiRequest,
    pub payload: UiPayload
}
#[derive(Debug, Clone)]
pub enum UiPayload {
    DetectedMidiPorts {
        ports: MidiPorts
    }
}


/// Runs in its own thread and coordinates all backend communication tasks.
pub struct UiBackend {

}

pub struct UiApi {
    pub input: Bus<UiRequest>,
    output_bus: Arc<Mutex<Bus<UiResponse>>>
    //pub output: BusReader<UiResponse>
}

impl UiApi {
    pub fn new_response_reader(&mut self) -> BusReader<UiResponse> {
        let mut output_bus = self.output_bus.lock().unwrap();
        output_bus.add_rx()
    }
}

impl Drop for UiApi {
    fn drop(&mut self) {
        // kill the thread
        self.input.broadcast(UiRequest::new(UiCommand::Drop))
    }
}

impl UiBackend {
    fn new() -> Self {
        UiBackend {

        }
    }
    pub fn spawn() -> UiApi {
        //let ui = UiBackend::new();

        let mut bus_output = Bus::new(16);
        let mut bus_output = Arc::new(Mutex::new(bus_output));

        let mut bus_input = Bus::new(16);
        let mut bus_input_rx = bus_input.add_rx();
        //let bus_output_rx = bus_output.add_rx();

        let api = UiApi {
            input: bus_input,
            output_bus: bus_output.clone()
        };

        thread::spawn(move || {
            let tick = Duration::from_millis(50);

            loop {
                if let Ok(request) = bus_input_rx.recv_timeout(tick) {
                    // process

                    match request.command {
                        UiCommand::ListMidiPorts => {
                            println!("ports you want?");

                            {
                                let midi = Midi::new();
                                if let Ok(midi_ports) = midi.detect_midi_ports() {
                                    bus_output.lock().unwrap().broadcast(UiResponse {
                                        request: request,
                                        payload: UiPayload::DetectedMidiPorts {
                                            ports: midi_ports
                                        }
                                    });

                                    println!("replied with ports!");
                                }
                            }

                        }
                        UiCommand::Drop => {
                            break;
                        }
                    }
                }
            }

            println!("shutting down");
        });

        api
    }
}