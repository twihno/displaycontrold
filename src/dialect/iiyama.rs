use std::net::{IpAddr, TcpStream};

use serialport::SerialPort;

use crate::{
    common::{self, FunctionConversionError},
    connection::{
        ConnectionType, SerialBaudrate, SerialConnectionParameters, SerialPortConnectionError,
        TcpConnectionParameters,
    },
};

#[derive(Debug)]
pub struct IiyamaDisplayController {
    connection: ConnectionType,
}

impl IiyamaDisplayController {
    /// Create a new `IiyamaDisplayController` with a serial connection
    pub fn new_serial(
        port_name: &str,
        baud_rate: SerialBaudrate,
        timeout: u64,
    ) -> Result<Self, SerialPortConnectionError> {
        let port = SerialConnectionParameters::new(
            Some(port_name.into()),
            Some(baud_rate),
            Some(serialport::DataBits::Eight),
            Some(serialport::StopBits::One),
            Some(serialport::Parity::None),
            Some(serialport::FlowControl::None),
            Some(std::time::Duration::from_secs(timeout)),
        )
        .connect()?;

        Ok(Self {
            connection: ConnectionType::Serial(port),
        })
    }

    /// Create a new [`IiyamaDisplayController`] with a serial connection and default settings.
    ///
    /// # Defaults
    /// - Baudrate: 9600
    /// - Timeout: 5 seconds
    pub fn new_serial_with_defaults(port_name: &str) -> Result<Self, SerialPortConnectionError> {
        Self::new_serial(port_name, SerialBaudrate::default(), 5)
    }

    pub fn new_tcp(ip: &IpAddr) -> std::io::Result<Self> {
        let config =
            TcpConnectionParameters::new(*ip, 5000, Some(std::time::Duration::from_secs(10)));
        let stream = config.connect()?;

        Ok(Self {
            connection: ConnectionType::Tcp(stream),
        })
    }
}

#[derive(Debug, Clone)]
struct RequestPackage {
    header: u8,
    monitor_id: u8,
    category: u8,
    code0: u8, // Page
    code1: u8, // Function
    length: u8,
    data_control: u8,
    data: Option<Vec<u8>>,
    checksum: u8,
}

impl RequestPackage {
    #[must_use]
    fn new(monitor_id: u8, function_code: u8, data: &Option<Vec<u8>>) -> Self {
        let length = data.as_ref().map_or(0, |d| d.len() as u8) + 3;

        let data = vec![function_code]
            .into_iter()
            .chain(data.as_ref().map_or(vec![], |d| d.clone()))
            .collect::<Vec<u8>>();

        let checksum = 0xa6 ^ monitor_id ^ length ^ 0x01 ^ data.iter().fold(0, |acc, &b| acc ^ b);

        Self {
            header: 0xa6,
            monitor_id,
            category: 0x00, // Always(?) for iiyama
            code0: 0x00,
            code1: 0x00,
            length,
            data_control: 0x01,
            data: Some(data.clone()),
            checksum,
        }
    }
}

/// Enum representing the different request functions for getting data from the monitor
#[derive(Debug, Clone, Copy)]
pub enum GetCommand {
    CommunicationControl,
    PlatformAndVersionLabels,
    PowerState,
    UserInputControl,
    PowerStateAtColdStart,
    CurrentSource,
    VideoParameters,
    ColorTemperature,
    ColorParameters,
    PictureFormat,
    Volume,
    AudioParameters,
    MiscellaneousInfo,
    SerialCode,
}

impl GetCommand {
    fn get_command_code(&self) -> u8 {
        match self {
            GetCommand::CommunicationControl => 0x00,
            GetCommand::PlatformAndVersionLabels => 0xa2,
            GetCommand::PowerState => 0x19,
            GetCommand::UserInputControl => 0x1d,
            GetCommand::PowerStateAtColdStart => 0xa4,
            GetCommand::CurrentSource => 0xad,
            GetCommand::VideoParameters => 0x33,
            GetCommand::ColorTemperature => 0x35,
            GetCommand::ColorParameters => 0x37,
            GetCommand::PictureFormat => 0x3b,
            GetCommand::Volume => 0x45,
            GetCommand::AudioParameters => 0x43,
            GetCommand::MiscellaneousInfo => 0x0f,
            GetCommand::SerialCode => 0x15,
        }
    }

    fn from_str(command: &str) -> Result<Self, FunctionConversionError> {
        // TODO
        todo!("Not implemented yet");
        // Err(FunctionConversionError::UnknownCommand)
    }
}

/// Enum representing the different request functions for setting data on the monitor
#[derive(Debug, Clone, Copy)]
pub enum SetCommand {
    CommunicationControl,
    PowerState(common::PowerState),
    UserInputControl,
    PowerStateAtColdStart,
    InputSource,
    VideoParameters,
    ColorTemperature,
    ColorParameters,
    PictureFormat,
    Volume,
    VolumeLimits,
    AudioParameters,
    AutoAdjust,
}

impl SetCommand {
    #[must_use]
    /// Returns the command code associated with the request function
    fn get_command_code(&self) -> u8 {
        match self {
            SetCommand::CommunicationControl => 0x00,
            SetCommand::PowerState(_) => 0x18,
            SetCommand::UserInputControl => 0x1c,
            SetCommand::PowerStateAtColdStart => 0xa3,
            SetCommand::InputSource => 0xac,
            SetCommand::VideoParameters => 0x32,
            SetCommand::ColorTemperature => 0x34,
            SetCommand::ColorParameters => 0x36,
            SetCommand::PictureFormat => 0x3a,
            SetCommand::Volume => 0x44,
            SetCommand::VolumeLimits => 0xb8,
            SetCommand::AudioParameters => 0x42,
            SetCommand::AutoAdjust => 0x70,
        }
    }

    pub fn from_str(command: &str, value: &str) -> Result<Self, FunctionConversionError> {
        match command {
            "power" => Ok(SetCommand::PowerState(value.try_into()?)),
            "input" => Ok(SetCommand::InputSource), // Placeholder for input source handling
            _ => Err(FunctionConversionError::UnknownCommand),
        }
    }

    #[must_use]
    fn get_payload_data(&self) -> Option<Vec<u8>> {
        match self {
            SetCommand::PowerState(state) => Some(vec![match state {
                common::PowerState::Off => 0x01,
                common::PowerState::On => 0x02,
            }]),
            _ => None,
        }
    }
}

pub struct GetRequest {
    pub monitor_id: u8,
    pub command: GetCommand,
}

fn construct_set_data_package(monitor_id: u8, function: SetCommand) -> RequestPackage {
    let command_code = function.get_command_code();
    let data = function.get_payload_data();

    RequestPackage::new(monitor_id, command_code, &data)
}

pub fn set(monitor_id: u8, function: SetCommand, port: &mut Box<dyn SerialPort>) {
    let package = construct_set_data_package(monitor_id, function);
    port.write_all(&Vec::<u8>::from(package))
        .expect("Failed to write to port");
}

impl From<RequestPackage> for Vec<u8> {
    fn from(package: RequestPackage) -> Self {
        let mut data = vec![
            package.header,
            package.monitor_id,
            package.category,
            package.code0,
            package.code1,
            package.length,
            package.data_control,
        ];
        if let Some(ref d) = package.data {
            data.extend_from_slice(d);
        }
        data.push(package.checksum);
        data
    }
}

pub enum RawResponseStatus<T> {
    Acknowledged(T),
    NotAcknowledged,
    NotAvailable,
}

pub type RawSetReponseStatus = RawResponseStatus<()>;
pub type RawGetResponseStatus<T> = RawResponseStatus<T>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the creation of a RawRequestPackage with simple data.
    #[test]
    fn test_raw_request_package() {
        let package = RequestPackage::new(1, 0x19, &Some(vec![0x01]));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 1);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 4);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x19, 0x01]);
        assert_eq!(package.checksum, 0xa6 ^ 1 ^ 4 ^ 0x01 ^ 0x19 ^ 0x01);
    }

    /// Tests the creation of a RawRequestPackage with no data.
    #[test]
    fn test_raw_request_package_no_data() {
        let package = RequestPackage::new(1, 0x19, &None);
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 1);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 3);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data, Some(vec![0x19]));
        assert_eq!(package.checksum, 0xa6 ^ 1 ^ 3 ^ 1 ^ 0x19);
    }

    #[test]
    fn test_set_power_state_off() {
        let package =
            construct_set_data_package(1, SetCommand::PowerState(common::PowerState::Off));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 1);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x18, 0x01]);
        assert_eq!(package.checksum, 0xbb);
    }

    #[test]
    fn test_set_power_state_on() {
        let package = construct_set_data_package(1, SetCommand::PowerState(common::PowerState::On));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 1);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x18, 0x02]);
        assert_eq!(package.checksum, 0xb8);
    }
}
