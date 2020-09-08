#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use serde::Deserialize;
use wmi::{COMLibrary, Variant, WMIConnection, WMIDateTime};
use std::collections::HashMap;

#[test]
fn list_serial_ports() -> Result<(), Box<dyn std::error::Error>>  {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT * FROM Win32_SerialPort")?;

    for os in results {
        println!("{:#?}", os);
    }

    #[derive(Deserialize, Debug)]
    struct Win32_SerialPort {
        Name: String,
        DeviceId: String,
        Description: String
    }

    let results: Vec<Win32_SerialPort> = wmi_con.query()?;

    for port in results {
        println!("{:#?}", port);
    }

    Ok(())
}