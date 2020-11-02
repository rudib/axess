use crate::FractalCoreError;
use ::fractal_protocol::{model::{FractalModel}};

use midir::{MidiInput, MidiOutput, Ignore, MidiInputConnection, MidiOutputConnection};
use log::{error, trace};
use super::{TransportMessage, Transport, TransportConnection};
use crossbeam_channel::Receiver;

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

pub struct MidiConnection {
    pub input: MidiInputConnection<()>,
    pub output: MidiOutputConnection,
    rx: Receiver<TransportMessage>
}


impl Transport for Midi {
    fn id(&self) -> String {
        "midi".into()
    }

    fn detect_endpoints(&self) -> Result<Vec<super::TransportEndpoint>, FractalCoreError> {
        let ports = self.detect_midi_ports()?;
        let devices = ports.detect_fractal_devices();
        let res = devices.into_iter().map(|d| {
            super::TransportEndpoint {
                id: format!("{}||{}", d.input_port_name, d.output_port_name),
                name: if let Some(fractal_model) = d.fractal_model {
                    format!("{}", fractal_model)
                } else {
                    format!("{} / {}", d.input_port_name, d.output_port_name)
                }
            }
        }).collect();
        Ok(res)
    }

    fn connect(&self, endpoint: &super::TransportEndpoint) -> Result<Box<dyn TransportConnection>, FractalCoreError> {
        let split = endpoint.id.split("||").collect::<Vec<_>>();
        if let (Some(input_port_name), Some(output_port_name)) = (split.get(0), split.get(1)) {

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


                    let (tx, rx) = crossbeam_channel::unbounded();

                    let in_connection = midi_in.connect(&midi_in_port, &format!("{} Axess In", &self.client_name), move |stamp, message, _data| {
                        trace!("MIDI input: {}: {:x?} (len = {})", stamp, message, message.len());
                        let tx = tx.send(message.to_vec());
                        if let Err(err) = tx {
                            error!("Error sending MIDI's input to channel: {:?}", err);
                        }
                    }, ())?;

                    let out_connection = midi_out.connect(&midi_out_port, &format!("{} Axess Out", &self.client_name))?;

                    trace!("MIDI connection initialized");

                    return Ok(Box::new(MidiConnection {
                        input: in_connection,
                        output: out_connection,
                        rx
                    }));
                },
                _ => {
                    trace!("Midi ports to connect not found!");
                    return Err(FractalCoreError::Unknown);
                }
            }
        } else {
            Err(FractalCoreError::Other("Split error?".into()))
        }
    }
}

impl TransportConnection for MidiConnection {
    fn get_receiver(&self) -> &Receiver<TransportMessage> {
        &self.rx
    }

    fn write(&mut self, buf: &[u8]) -> crate::FractalResultVoid {
        trace!("Writing to MIDI port: {:X?}", &buf);
        self.output.send(buf)?;
        Ok(())
    }
}