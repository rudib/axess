#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os="windows")]
extern crate wmi;

extern crate serialport;

use crate::{FractalResultVoid, FractalCoreError};
use std::{time::Duration, io::{Write, self}, sync::{Mutex, Arc}};
use super::{TransportConnection, Transport, TransportEndpoint, TransportMessage};
use crossbeam_channel::{Receiver};
use io::{BufReader, BufRead};
use serialport::SerialPort;
use fractal_protocol::buffer::SYSEX_END;
use log::{error, trace};

#[derive(Debug, Clone)]
pub struct DetectedSerialPort {
    pub name: String,
    pub port: String
}

#[cfg(target_os="windows")]
fn detect_serial_ports() -> Vec<DetectedSerialPort> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};
    
    fn wmi_detect() -> Result<Vec<DetectedSerialPort>, FractalCoreError> {
        #[derive(Deserialize, Debug)]
        struct Win32_SerialPort {
            Name: String,
            DeviceId: String,
            Description: String
        }

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con.into())?;
        let results = wmi_con.query::<Win32_SerialPort>()?;
        
        let ret = results.into_iter().map(|r| {
            DetectedSerialPort {
                name: r.Name,
                port: r.DeviceId
            }
        }).collect();
        Ok(ret)
    }

    match wmi_detect() {
        Ok(ret) => { return ret; }
        Err(e) => {
            error!("WMI port detection failed: {:?}", e);
        }
    }

    match serialport::available_ports() {
        Ok(ret) => {
            let ret = ret.iter().map(|p| DetectedSerialPort {
                name: p.port_name.clone(),
                port: p.port_name.clone()
            }).collect();
            return ret;
        },
        Err(e) => {
            error!("serialport-rs port detection failed: {:?}", e);
        }
    }

    vec![]
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

#[cfg(target_os="windows")]
impl From<wmi::WMIError> for FractalCoreError {
    fn from(e: wmi::WMIError) -> Self {
        FractalCoreError::Other(format!("WMI Error: {:?}", e))
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
        let timeout = Duration::from_millis(20);
        port.set_timeout(timeout)?;

        let (serial_read_tx, serial_read_rx) = crossbeam_channel::unbounded();

        let stop = Arc::new(Mutex::new(false));

        {
            let port = port.try_clone()?;
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut buffered_reader = BufReader::new(port);                
                loop {
                    let mut buffer = vec![];
                    match buffered_reader.read_until(SYSEX_END, &mut buffer) {
                        Ok(bytes) => {
                            let buffer = &buffer[0..bytes];
                            match serial_read_tx.send(buffer.to_vec()) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Serial port read sending to channel failure: {:?}", e);
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => {
                            error!("Serial port read failure: {:?}", e);
                            if let Ok(mut stop) = stop.lock() {
                                *stop = true;
                            }
                        }
                    }

                    if let Ok(stop) = stop.try_lock() {
                        if *stop == true {
                            trace!("serial stop mutex");
                            break;
                        }
                    }
                }

                trace!("shutdown serial read thread");
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
        trace!("Writing to serial port: {:X?}", buf);
        self.port.write(buf)?;
        Ok(())
    }
}

impl Drop for TransportSerialConnection {
    fn drop(&mut self) {
        trace!("Dropping serial connection");
        if let Ok(mut stop) = self.stop.lock() {
            *stop = true;
        }
    }
}


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