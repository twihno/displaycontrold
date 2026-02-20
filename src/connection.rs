use core::time;
use std::net::{IpAddr, TcpStream};

use serialport::SerialPort;
use thiserror::Error;

const DEFAULT_CONNECTION_TIMEOUT: time::Duration = time::Duration::from_secs(10);

#[derive(Error, Debug)]
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

#[derive(Debug)]
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
            port: port.unwrap_or("/dev/ttyUSB0".into()),
            baud_rate: baud_rate.unwrap_or(SerialBaudrate::B9600),
            data_bits,
            stop_bits,
            parity_bit,
            flow_control,
            timeout: timeout.unwrap_or(DEFAULT_CONNECTION_TIMEOUT),
        }
    }

    /// Establishes a new serial connection with the provided settings with a fallback to sensible defaults.
    pub fn connect(&self) -> Result<Box<dyn SerialPort>, SerialPortConnectionError> {
        let mut port = serialport::new(&self.port, (&self.baud_rate).into())
            .open()
            .map_err(|err| SerialPortConnectionError::OpenError(err))?;

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

#[derive(Debug)]
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
    #[error("Unsupported baudrate")]
    UnsupportedBaudrate,
}

impl From<&SerialBaudrate> for u32 {
    fn from(baudrate: &SerialBaudrate) -> Self {
        match baudrate {
            SerialBaudrate::B1200 => 1200,
            SerialBaudrate::B2400 => 2400,
            SerialBaudrate::B4800 => 4800,
            SerialBaudrate::B9600 => 9600,
            SerialBaudrate::B19200 => 19200,
            SerialBaudrate::B38400 => 38400,
            SerialBaudrate::B57600 => 57600,
        }
    }
}

impl TryFrom<u32> for SerialBaudrate {
    type Error = SerialBaudrateError;

    /// Try to convert a u32 value into a SerialBaudrate
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
            _ => Err(SerialBaudrateError::UnsupportedBaudrate),
        }
    }
}
