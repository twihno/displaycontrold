use std::{net::IpAddr, process::exit};

use serde::Deserialize;

use crate::controller::UntypedSettingEntry;

pub mod common;
pub mod connection;
pub mod controller;
pub mod dialect;

#[derive(Debug, Clone, Deserialize)]
/// The low level settings for the actual connection to the display
pub enum ConnectionSettings {
    #[serde(rename = "serial")]
    Serial {
        port: String,
        baud_rate: Option<u32>,
        flow_control: Option<serialport::FlowControl>,
        parity_bit: Option<serialport::Parity>,
        timeout: Option<u64>,
    },
    #[serde(rename = "tcp")]
    Tcp {
        ip: IpAddr,
        port: u16,
        timeout: Option<u64>,
    },
}

#[derive(Debug, Clone, Deserialize)]
///
pub struct ReadSettings {
    connection: ConnectionSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteSettings {
    connection: ConnectionSettings,
    settings: Vec<UntypedSettingEntry>,
}

pub fn apply_settings(settings: Vec<WriteSettings>) {
    if settings.len() == 0 {
        eprintln!("No settings provided to apply.");
        exit(0);
    }

    let mut control_threads = Vec::new();

    for setting in settings {
        control_threads.push(std::thread::spawn(move || {}));
    }

    for thread in control_threads {
        if let Err(e) = thread.join() {
            eprintln!("A control thread panicked: {:?}", e);
        }
    }
}
