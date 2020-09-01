mod fractal;

extern crate midir;
extern crate bus;


use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::sync::Mutex;
use std::{time::Duration, thread};

use midir::{MidiInput, MidiOutput, Ignore, MidiInputPort, MidiOutputPort, MidiInputConnection, MidiOutputConnection};
use bus::Bus;

use fractal::common::*;
use fractal::model::*;


fn main() {
    //println!("Hello, world!");


    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }

    
}



fn run() -> Result<(), Box<dyn Error>> {

    let mut midi_in = MidiInput::new("midir test input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir test output")?;

    let mut input = String::new();

    /*
    println!("Available input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
    }
    
    println!("\nAvailable output ports:");
    for (i, p) in midi_out.ports().iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }
    */

    let midi_in_ports = midi_in.ports();
    let midi_out_ports = midi_out.ports();

    let axe_in = midi_in_ports.iter().find(|&p| midi_in.port_name(p).map_or(false, |p| p.contains("MIDI In")));
    let axe_out = midi_out_ports.iter().find(|&p | midi_out.port_name(p).map_or(false, |p| p.contains("MIDI Out")));
    
    match (axe_in, axe_out) {
        (Some(axe_in), Some(axe_out)) => {


            println!("Found some midi ports!");

            let connection = FractalConnection::connect(midi_in, axe_in, midi_out, axe_out).unwrap();





        },
        _ => { println!("Not found!"); }
    }
    
    Ok(())
}

struct FractalConnection {
    input: MidiInputConnection<()>,
    output: MidiOutputConnection,
    model: FractalModel,
    firmware: (u8, u8)
}

/*
const static byte SYSEX_MANUFACTURER_BYTE1 = 0x00;
const static byte SYSEX_MANUFACTURER_BYTE2 = 0x01;
const static byte SYSEX_MANUFACTURER_BYTE3 = 0x74;
const static byte SYSEX_AXE_VERSION = 0x10;
const static byte SYSEX_QUERY_BYTE = 0x7F;
const static byte SYSEX_CHECKSUM_PLACEHOLDER = 0x00;
*/
const SYSEX_MANUFACTURER_BYTE1: u8 = 0x00;
const SYSEX_MANUFACTURER_BYTE2: u8 = 0x01;
const SYSEX_MANUFACTURER_BYTE3: u8 = 0x74;

impl FractalConnection {
    pub fn connect(midi_in: MidiInput, midi_in_port: &MidiInputPort, midi_out: MidiOutput, midi_out_port: &MidiOutputPort) -> Result<Self, FrakoError> {
        
        let mut bus = Bus::new(20);
        let mut r = bus.add_rx();
        let mut r_debug = bus.add_rx();

        let mut buffer = Mutex::new(vec![]);
        let in_connection = midi_in.connect(midi_in_port, "midir-in", move |stamp, message, _| {
            
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            
            if let Ok(mut buffer) = buffer.lock() {
                for b in message {
                    buffer.push(*b);
                    if *b == 0xF7 {
                        // sysex message end, do a checksum test
                        let mut valid_msg = false;

                        if buffer.len() > 5 && 
                           buffer[0] == 0xF0 &&
                           buffer[1] == SYSEX_MANUFACTURER_BYTE1 &&
                           buffer[2] == SYSEX_MANUFACTURER_BYTE2 &&
                           buffer[3] == SYSEX_MANUFACTURER_BYTE3 &&
                           buffer[buffer.len() - 1] == 0xF7 
                        {
                            let crc_byte = buffer[buffer.len() - 2];
                            //println!("crc_byte: {}", crc_byte);
                            let calc = calc_checksum(&buffer[..buffer.len() - 2]);
                            //println!("calc: {}", calc);

                            if calc == crc_byte {
                                valid_msg = true;
                            }
                        }

                        // decode the sysex message
                        if (valid_msg) {
                            let fractal_msg = fractal::message::parse_message(&buffer);
                            bus.broadcast(fractal_msg);
                        }

                        buffer.clear();
                    }
                }
            }

        }, ()).unwrap();

        {
            // debugger
            thread::spawn(move || {
                loop {
                    if let Ok(msg) = r_debug.recv() {
                        println!("parsed message: {:?}", msg);
                    }
                }
            });
        }

        let mut out_connection = midi_out.connect(midi_out_port, "midir-out").unwrap();

        let (mut firmware_major, mut firmware_minor, mut model) = (None, None, None);
        
        // detect the model
        out_connection.send(&wrap_msg(vec![0x7F, 0x00])).unwrap();
        if let Ok(msg) = r.recv_timeout(Duration::from_millis(500)) {
            if let Some(msg_model) = msg.model {
                model = Some(msg_model);
            }
        }

        if model == None {
            return Err(FrakoError::CompatibleNotDetected);
        }

        // detect the firmware
        out_connection.send(&get_firmware_version(model_code(model.unwrap()))).unwrap();
        if let Ok(msg) = r.recv_timeout(Duration::from_millis(500)) {
            match msg.message {
                fractal::message::FractalMessage::FirmwareVersion{ major, minor } => {
                    firmware_major = Some(major);
                    firmware_minor = Some(minor);
                },
                _ => ()
            }
        }

        if firmware_major == None && firmware_minor == None {
            return Err(FrakoError::CompatibleNotDetected);
        }

        out_connection.send(&get_current_preset_name(model.unwrap())).unwrap();

        std::thread::sleep_ms(15000);

        out_connection.send(&disconnect_from_controller(model.unwrap())).unwrap();

        //std::thread::sleep_ms(0);

        Ok(FractalConnection {
            input: in_connection,
            output: out_connection,
            model: FractalModel::III,
            firmware: (firmware_major.unwrap(), firmware_minor.unwrap())
        })
    }


}

#[derive(Debug)]
enum FrakoError {
    Unknown,    
    CompatibleNotDetected
}

pub fn get_current_preset_name(model: FractalModel) -> MidiMessage {
    if model == FractalModel::III {
        wrap_msg(vec![model_code(model), 0x0D, 0x7F, 0x7F])
    } else {
        wrap_msg(vec![model_code(model), 0x0F])
    }
}

pub fn get_firmware_version(model_code: u8) -> MidiMessage {
    wrap_msg(vec![model_code, 0x08])
}

pub fn disconnect_from_controller(model: FractalModel) -> MidiMessage {
    wrap_msg(vec![model_code(model), 0x42])
}