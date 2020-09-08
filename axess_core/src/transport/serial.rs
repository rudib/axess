#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os="windows")]
extern crate wmi;

extern crate serialport;

use crate::{FractalResultVoid, FractalCoreError};
use std::{time::Duration, io::{Read, Write, self}, sync::{Mutex, Arc}};
use super::{TransportConnection, Transport, TransportEndpoint, TransportMessage};
use crossbeam_channel::{Receiver};
use serialport::SerialPort;

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

#[cfg(not(target_os="windows"))]
pub fn detect_serial_ports() -> Vec<SerialPort> {
    not_implemented!("Missing support for serial ports.");
}

impl From<serialport::Error> for FractalCoreError {
    fn from(_: serialport::Error) -> Self {
        FractalCoreError::IoError
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
    fn detect_endpoints(&self) -> Result<Vec<super::TransportEndpoint>, FractalCoreError> {
        let ports = detect_serial_ports();
        let endpoints = ports.iter().map(|p| TransportEndpoint {
            id: p.port.clone(),
            name: p.name.clone()
        }).collect();
        Ok(endpoints)
    }

    fn connect(&self, endpoint: &super::TransportEndpoint) -> Result<Box<dyn TransportConnection>, FractalCoreError> {
        let mut port = serialport::open(&endpoint.id)?;
        let timeout = Duration::from_millis(100);
        port.set_timeout(timeout)?;

        //let (serial_write_tx, serial_write_rx) = crossbeam_channel::unbounded::<TransportMessage>();
        let (serial_read_tx, serial_read_rx) = crossbeam_channel::unbounded();

        let stop = Arc::new(Mutex::new(false));

        let read_thread = {
            let mut port = port.try_clone()?;
            let mut stop = stop.clone();
            std::thread::spawn(move || {
                let mut buffer = [0; 512];
                loop {
                    match port.read(&mut buffer) {
                        Ok(bytes) => {
                            match serial_read_tx.send(buffer[0..bytes].to_vec()) {
                                Ok(_) => (),
                                Err(e) => {
                                    eprintln!("Sending from TX: {:?}", e);
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => {
                            eprintln!("{:?}", e);
                            if let Ok(mut stop) = stop.lock() {
                                *stop = true;
                            }
                        }
                    }

                    if let Ok(stop) = stop.try_lock() {
                        if *stop == true {
                            break;
                        }
                    }
                }

                println!("shutdown read thread");
            })
        };

        Ok(Box::new(TransportSerialConnection {
            port,
            serial_read_rx,
            stop
        }))
    }

    fn id(&self) -> String {
        "serial".into()
    }
}

pub struct TransportSerialConnection {
    port: Box<dyn SerialPort>,
    serial_read_rx: Receiver<TransportMessage>,
    stop: Arc<Mutex<bool>>
}

impl TransportConnection for TransportSerialConnection {
    fn get_receiver(&self) -> &crossbeam_channel::Receiver<Vec<u8>> {
        &self.serial_read_rx
    }

    fn write(&mut self, buf: &[u8]) -> FractalResultVoid {
        self.port.write(buf)?;
        Ok(())
    }
}

impl Drop for TransportSerialConnection {
    fn drop(&mut self) {
        if let Ok(mut stop) = self.stop.lock() {
            *stop = true;
        }
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
#[ignore]
fn serial_test() -> Result<(), FractalCoreError> {
    let serial_transport = TransportSerial::new();
    let endpoints = serial_transport.detect_endpoints()?;
    println!("endpoints: {:?}", endpoints);

    let mut connection = serial_transport.connect(&endpoints[0])?;
    connection.write(b"ABC").unwrap();
    let received = connection.get_receiver().recv().unwrap();
    println!("received: {:?}", received);


    Ok(())
}