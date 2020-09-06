use super::FractalCoreError;

use ::fractal_protocol::{common::wrap_msg, model::{FractalDevice, FractalModel}};

use midir::{MidiInput, MidiOutput, Ignore, MidiInputPort, MidiOutputPort, MidiInputConnection, MidiOutputConnection};
use log::{trace};
use std::time::Duration;

pub struct Midi {
    client_name: String
}

impl Midi {
    pub fn new() -> Result<Self, FractalCoreError> {
        Ok(Midi {
            client_name: "FractalCore".into()
        })
    }

    pub fn detect_midi_ports(&self) -> Result<MidiPorts, FractalCoreError> {
        
        let mut midi_in = MidiInput::new(&format!("{} input", &self.client_name))?;
        midi_in.ignore(Ignore::None);
        let midi_out = MidiOutput::new(&format!("{} output", &self.client_name))?;

        let midi_in_ports = midi_in.ports();
        let midi_out_ports = midi_out.ports();

        let ports = MidiPorts {
            inputs: midi_in_ports.iter().filter_map(|p| midi_in.port_name(p).ok()).collect(),
            outputs: midi_out_ports.iter().filter_map(|p| midi_out.port_name(p).ok()).collect()
        };

        trace!("Detected MIDI ports: {:?}", ports);

        Ok(ports)
    }

    pub fn connect_to<T: Send, TFnCallback: 'static + Fn(&[u8], &mut T) + Send>(&self, input_port_name: &str, output_port_name: &str, callback: TFnCallback, callback_ctx: T) 
        -> Result<MidiConnection<T>, FractalCoreError>
    {
        
        let mut midi_in = MidiInput::new(&format!("{} input", &self.client_name))?;
        midi_in.ignore(Ignore::None);
        let midi_out = MidiOutput::new(&format!("{} output", &self.client_name))?;

        let midi_in_ports = midi_in.ports();
        let midi_out_ports = midi_out.ports();

        let midi_in_port = midi_in_ports.into_iter().find(|p| midi_in.port_name(p).ok().map(|n| &n == input_port_name).unwrap_or(false));
        let midi_out_port = midi_out_ports.into_iter().find(|p| midi_out.port_name(p).ok().map(|n| &n == output_port_name).unwrap_or(false));

        match (midi_in_port, midi_out_port) {
            (Some(midi_in_port), Some(midi_out_port)) => {
                trace!("Matched the MIDI ports");

                let in_connection = midi_in.connect(&midi_in_port, &format!("{} Axess In", &self.client_name), move |stamp, message, data| {
                    trace!("MIDI input: {}: {:x?} (len = {})", stamp, message, message.len());
                    callback(message, data);
                }, callback_ctx)?;

                let mut out_connection = midi_out.connect(&midi_out_port, &format!("{} Axess Out", &self.client_name))?;

                trace!("MIDI connection initialized");

                return Ok(MidiConnection {
                    input: in_connection,
                    output: out_connection
                });
            },
            _ => {
                trace!("Midi ports to connect not found!");
                return Err(FractalCoreError::Unknown);
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct MidiPorts {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>
}


impl MidiPorts {
    pub fn detect_fractal_devices(&self) -> Vec<MidiConnectionDeviceRequest> {
        // todo: this only works with a single device of one type connected to the MIDI adapter

        let mut ret = vec![];

        let known_names = [
            ("Axe-Fx III", FractalModel::III),
            ("FM3", FractalModel::FM3)
        ];

        for (name, model) in &known_names {
            let filter = |port_name: &&String| {
                port_name.contains(name)
            };

            let input_port = self.inputs.iter().find(filter);
            let output_port = self.outputs.iter().find(filter);

            match (input_port, output_port) {
                (Some(input_port), Some(output_port)) => {
                    ret.push(MidiConnectionDeviceRequest {
                        input_port_name: input_port.into(),
                        output_port_name: output_port.into(),
                        fractal_model: Some(*model)
                    })
                },
                _ => ()
            }
        }

        ret
    }
}
#[derive(Debug)]
pub struct MidiConnectionDeviceRequest {
    pub input_port_name: String,
    pub output_port_name: String,
    pub fractal_model: Option<FractalModel>
}

pub struct MidiConnection<T: 'static> {
    pub input: MidiInputConnection<T>,
    pub output: MidiOutputConnection
}