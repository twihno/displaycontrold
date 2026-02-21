use core::time;
use std::net::{IpAddr, TcpStream};

use serde::Deserialize;
use serialport::SerialPort;
use thiserror::Error;

const DEFAULT_CONNECTION_TIMEOUT: time::Duration = time::Duration::from_secs(10);
const DEFAULT_SERIAL_BAUDRATE: SerialBaudrate = SerialBaudrate::B9600;

#[cfg(target_os = "windows")]
const DEFAULT_SERIAL_PORT: &str = "COM1";
#[cfg(not(target_os = "windows"))]
const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";

#[derive(Debug, Clone, Deserialize)]
/// The low level settings for the actual connection to the display
pub enum UserConnectionSettings {
    #[serde(rename = "serial")]
    Serial {
        port: String,
        baud_rate: Option<SerialBaudrate>,
        timeout: Option<u64>,
    },
    #[serde(rename = "tcp")]
    Tcp {
        ip: IpAddr,
        port: u16,
        timeout: Option<u64>,
    },
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to establish a serial connection: {0}")]
    SerialConnectionError(SerialPortConnectionError),

    #[error("Failed to establish a TCP connection: {0}")]
    TcpConnectionError(std::io::Error),
}

#[derive(Debug, Error)]
pub enum SerialPortConnectionError {
    #[error("Failed to open serial port: {0}")]
    /// Failed to open serial port
    OpenError(serialport::Error),

    #[error("Failed to change settings on serial port: {0}")]
    SettingsError(serialport::Error),
}

impl SerialPortConnectionError {
    #[must_use]
    fn new_settings_error(err: serialport::Error) -> Self {
        SerialPortConnectionError::SettingsError(err)
    }
}

#[derive(Debug, Clone)]
pub struct SerialConnectionParameters {
    port: String,
    baud_rate: SerialBaudrate,
    data_bits: Option<serialport::DataBits>,
    stop_bits: Option<serialport::StopBits>,
    parity_bit: Option<serialport::Parity>,
    flow_control: Option<serialport::FlowControl>,
    timeout: time::Duration,
}

impl SerialConnectionParameters {
    #[must_use]
    pub fn new(
        port: Option<String>,
        baud_rate: Option<SerialBaudrate>,
        data_bits: Option<serialport::DataBits>,
        stop_bits: Option<serialport::StopBits>,
        parity_bit: Option<serialport::Parity>,
        flow_control: Option<serialport::FlowControl>,
        timeout: Option<time::Duration>,
    ) -> Self {
        Self {
            port: port.unwrap_or(DEFAULT_SERIAL_PORT.into()),
            baud_rate: baud_rate.unwrap_or(DEFAULT_SERIAL_BAUDRATE),
            data_bits,
            stop_bits,
            parity_bit,
            flow_control,
            timeout: timeout.unwrap_or(DEFAULT_CONNECTION_TIMEOUT),
        }
    }

    /// Replace all provided fields
    pub fn force(
        &mut self,
        port: Option<String>,
        baud_rate: Option<SerialBaudrate>,
        data_bits: Option<Option<serialport::DataBits>>,
        stop_bits: Option<Option<serialport::StopBits>>,
        parity_bit: Option<Option<serialport::Parity>>,
        flow_control: Option<Option<serialport::FlowControl>>,
        timeout: Option<time::Duration>,
    ) {
        if let Some(port) = port {
            self.port = port;
        }
        if let Some(baud_rate) = baud_rate {
            self.baud_rate = baud_rate;
        }
        if let Some(data_bits) = data_bits {
            self.data_bits = data_bits;
        }
        if let Some(stop_bits) = stop_bits {
            self.stop_bits = stop_bits;
        }
        if let Some(parity_bit) = parity_bit {
            self.parity_bit = parity_bit;
        }
        if let Some(flow_control) = flow_control {
            self.flow_control = flow_control;
        }
        if let Some(timeout) = timeout {
            self.timeout = timeout;
        }
    }

    #[must_use]
    pub fn is_valid_baud_rate(&self, allowed_values: &[SerialBaudrate]) -> bool {
        allowed_values.contains(&self.baud_rate)
    }

    /// Initialize a new serial connection with the previously set settings
    pub fn connect(&self) -> Result<Box<dyn SerialPort>, SerialPortConnectionError> {
        let mut port = serialport::new(&self.port, self.baud_rate as u32)
            .open()
            .map_err(SerialPortConnectionError::OpenError)?;

        if let Some(data_bits) = self.data_bits {
            // TODO: add debug logging
            port.set_data_bits(data_bits)
                .map_err(SerialPortConnectionError::new_settings_error)?;
        }

        if let Some(stop_bits) = self.stop_bits {
            // TODO: add debug logging
            port.set_stop_bits(stop_bits)
                .map_err(SerialPortConnectionError::new_settings_error)?;
        }

        if let Some(parity_bit) = self.parity_bit {
            // TODO: add debug logging
            port.set_parity(parity_bit)
                .map_err(SerialPortConnectionError::new_settings_error)?;
        }

        if let Some(flow_control) = self.flow_control {
            // TODO: add debug logging
            port.set_flow_control(flow_control)
                .map_err(SerialPortConnectionError::new_settings_error)?;
        }

        // TODO: add debug logging
        port.set_timeout(self.timeout)
            .map_err(SerialPortConnectionError::new_settings_error)?;

        // TODO: add debug logging
        port.clear(serialport::ClearBuffer::All)
            .map_err(SerialPortConnectionError::new_settings_error)?;

        Ok(port)
    }
}

#[derive(Debug, Clone)]
pub struct TcpConnectionParameters {
    ip: IpAddr,
    port: u16,
    timeout: time::Duration,
}

impl TcpConnectionParameters {
    #[must_use]
    pub fn new(ip: IpAddr, port: u16, timeout: Option<time::Duration>) -> Self {
        Self {
            ip,
            port,
            timeout: timeout.unwrap_or(DEFAULT_CONNECTION_TIMEOUT),
        }
    }

    /// Replace all provided fields
    pub fn force(
        &mut self,
        ip: Option<IpAddr>,
        port: Option<u16>,
        timeout: Option<time::Duration>,
    ) {
        if let Some(ip) = ip {
            self.ip = ip;
        }
        if let Some(port) = port {
            self.port = port;
        }
        if let Some(timeout) = timeout {
            self.timeout = timeout;
        }
    }

    /// Initiate a new TCP connection with the previously set settings
    pub fn connect(&self) -> std::io::Result<TcpStream> {
        let connection = TcpStream::connect((self.ip, self.port))?;

        // TODO: add debug logging
        connection.set_read_timeout(Some(self.timeout))?;
        connection.set_write_timeout(Some(self.timeout))?;

        Ok(connection)
    }
}

#[derive(Debug)]
/// Represents the underlying connection used to communicate with the display
///
/// The options in case of an iiyama display are RS-232 (serial) and TCP/IP.
pub enum ConnectionType {
    Serial(Box<dyn SerialPort>),
    Tcp(TcpStream),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SerialBaudrate {
    B1200 = 1200,
    B2400 = 2400,
    B4800 = 4800,
    #[default]
    B9600 = 9600,
    B19200 = 19200,
    B38400 = 38400,
    B57600 = 57600,
}

#[derive(Debug, Error)]
pub enum SerialBaudrateError {
    #[error("Unsupported baudrate: {0}")]
    UnsupportedBaudrate(u32),
}

impl TryFrom<u32> for SerialBaudrate {
    type Error = SerialBaudrateError;

    /// Try to convert a u32 value into a [`SerialBaudrate`]
    ///
    /// # Errors
    /// Returns [`SerialBaudrateError::UnsupportedBaudrate`]
    /// if the value doesn't match a supported baudrate.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1200 => Ok(SerialBaudrate::B1200),
            2400 => Ok(SerialBaudrate::B2400),
            4800 => Ok(SerialBaudrate::B4800),
            9600 => Ok(SerialBaudrate::B9600),
            19200 => Ok(SerialBaudrate::B19200),
            38400 => Ok(SerialBaudrate::B38400),
            57600 => Ok(SerialBaudrate::B57600),
            _ => Err(SerialBaudrateError::UnsupportedBaudrate(value)),
        }
    }
}

impl<'de> Deserialize<'de> for SerialBaudrate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        SerialBaudrate::try_from(value).map_err(serde::de::Error::custom)
    }
}
