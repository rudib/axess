use super::FractalCoreError;

use ::fractal_protocol::model::FractalModel;

use midir::{MidiInput, MidiOutput, Ignore, MidiInputPort, MidiOutputPort, MidiInputConnection, MidiOutputConnection};
use log::{trace};

pub struct Midi {
    client_name: String
}

impl Midi {
    pub fn new() -> Self {
        Midi {
            client_name: "FractalCore".into()
        }
    }

    pub fn detect_midi_ports(&self) -> Result<MidiPorts, FractalCoreError> {
        let mut midi_in = MidiInput::new(&format!("{} input", self.client_name))?;
        midi_in.ignore(Ignore::None);
        let midi_out = MidiOutput::new(&format!("{} output", self.client_name))?;

        let midi_in_ports = midi_in.ports();
        let midi_out_ports = midi_out.ports();

        let ports = MidiPorts {
            inputs: midi_in_ports.iter().filter_map(|p| midi_in.port_name(p).ok()).collect(),
            outputs: midi_out_ports.iter().filter_map(|p| midi_out.port_name(p).ok()).collect()
        };

        trace!("Detected MIDI ports: {:?}", ports);

        Ok(ports)
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
                port_name.starts_with(name)
            };

            let input_port = self.inputs.iter().find(filter);
            let output_port = self.outputs.iter().find(filter);

            match (input_port, output_port) {
                (Some(input_port), Some(output_port)) => {
                    ret.push(MidiConnectionDeviceRequest {
                        input_port_name: input_port.into(),
                        output_port_name: output_port.into(),
                        fractal_model: *model
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
    pub fractal_model: FractalModel
}
