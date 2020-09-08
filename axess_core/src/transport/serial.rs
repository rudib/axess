#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os="windows")]
extern crate wmi;

extern crate serial;

use crate::FractalCoreError;
use std::{time::Duration, io::{Read, Write, ErrorKind}, thread::JoinHandle};
use serial::{windows::COMPort, SerialPort};
use super::{TransportConnection, Transport, TransportEndpoint, TransportMessage};
use crossbeam_channel::{Sender, Receiver};

#[derive(Debug, Clone)]
pub struct DetectedSerialPort {
    pub name: String,
    pub port: String
}

#[cfg(target_os="windows")]
fn detect_serial_ports() -> Vec<DetectedSerialPort> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    let mut ret = vec![];

    #[derive(Deserialize, Debug)]
    struct Win32_SerialPort {
        Name: String,
        DeviceId: String,
        Description: String
    }

    if let Ok(com_con) = COMLibrary::new() {
        if let Ok(wmi_con) = WMIConnection::new(com_con.into()) {
            if let Ok(results) = wmi_con.query::<Win32_SerialPort>() {
                for r in results {
                    ret.push(DetectedSerialPort {
                        name: r.Name,
                        port: r.DeviceId
                    })
                }
            }
        }
    }

    ret
}

pub fn port_test(port_name: &str) -> Result<(), FractalCoreError> {
    let mut port = serial::open(port_name)?;
    println!("open!");
    
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(100))?;
    println!("configured");
    
    port.write(&['A' as u8, 'B' as u8, 'C' as u8])?;
    println!("written");

    let mut buf = [0; 150];
    let len = port.read(&mut buf)?;
    println!("read {} bytes", len);

    match port.read(&mut buf) {
        Ok(len) => {

        },
        Err(e) if e.kind() == ErrorKind::TimedOut => {
            println!("timed out, retry");
        },
        Err(e) => {
            println!("error: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(not(target_os="windows"))]
pub fn detect_serial_ports() -> Vec<SerialPort> {
    not_implemented!("Missing support for serial ports.");
}

impl From<serial::Error> for FractalCoreError {
    fn from(_: serial::Error) -> Self {
        FractalCoreError::Other("Serial IO".into())
    }
}


pub struct TransportSerial {

}

impl TransportSerial {
    pub fn new() -> Self {
        TransportSerial {

        }
    }
}

impl Transport for TransportSerial {
    type TConnection = TransportSerialConnection;

    fn detect_endpoints(&self) -> Result<Vec<super::TransportEndpoint>, FractalCoreError> {
        let ports = detect_serial_ports();
        let endpoints = ports.iter().map(|p| TransportEndpoint {
            id: p.port.clone(),
            name: p.name.clone()
        }).collect();
        Ok(endpoints)
    }

    fn connect(&self, endpoint: &super::TransportEndpoint) -> Result<Self::TConnection, FractalCoreError> {
        let mut port = serial::open(&endpoint.id)?;
    
        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;

        port.set_timeout(Duration::from_millis(100))?;

        let (serial_write_tx, serial_write_rx) = crossbeam_channel::unbounded::<TransportMessage>();
        let (serial_read_tx, serial_read_rx) = crossbeam_channel::unbounded();

        let io_thread = std::thread::spawn(move || {
            loop {
                let t = Duration::from_millis(5);

                if let Ok(msg) = serial_write_rx.recv_timeout(t) {
                    port.write(&msg);
                }

                let mut buf = [0;512];
                match port.read(&mut buf) {
                    Ok(len) => {
                        serial_read_tx.send(buf[0..len].to_vec());
                    },
                    Err(e) if e.kind() == ErrorKind::TimedOut => {

                    },
                    Err(e) => {
                        println!("some error: {:?}", e);
                    }
                }
            }
        });

        Ok(TransportSerialConnection {
            serial_write_tx,
            serial_read_rx,
            io_thread
        })
    }

    fn id() -> String {
        "serial".into()
    }
}

pub struct TransportSerialConnection {
    serial_write_tx: Sender<TransportMessage>,
    serial_read_rx: Receiver<TransportMessage>,
    io_thread: JoinHandle<()>
}

impl TransportConnection for TransportSerialConnection {
    fn get_receiver(&self) -> &crossbeam_channel::Receiver<Vec<u8>> {
        &self.serial_read_rx
    }

    fn get_sender(&self) -> &crossbeam_channel::Sender<Vec<u8>> {
        &self.serial_write_tx
    }
}





/*
#[test]
fn list_serial_ports() {
    let ports = detect_serial_ports();
    println!("ports: {:?}", ports);

    if ports.len() == 1 {
        port_test(&ports.first().unwrap().port).unwrap();
    }    
}
*/

#[test]
fn transport() -> Result<(), FractalCoreError> {
    let serial_transport = TransportSerial::new();
    let endpoints = serial_transport.detect_endpoints()?;
    println!("endpoints: {:?}", endpoints);

    let connection = serial_transport.connect(&endpoints[0])?;
    connection.get_sender().send(b"ABC".to_vec()).unwrap();
    let received = connection.get_receiver().recv().unwrap();
    println!("received: {:?}", received);


    Ok(())
}