use serde::Deserialize;
use serialport::SerialPort;

use crate::{
    connection::{
        ConnectionError, ConnectionType, SerialBaudrate, SerialConnectionParameters,
        SerialPortConnectionError, TcpConnectionParameters, UserConnectionSettings,
    },
    controller::{
        AddSettingsError, DisplayController, ExecuteSettingsError, SettingDiff, SettingEntry,
    },
    dialect::iiyama::{
        get_commands::GetCommand,
        set_commands::{
            AudioParameters, BlockedUserInput, ColorParameters, ColorTemperature, InputSource,
            PictureFormat, PowerState, PowerStateAtColdStart, SetCommand, VideoParameters, Volume,
            VolumeLimits,
        },
    },
};

mod get_commands;
mod set_commands;

const SUPPORTED_BAUD_RATES: [SerialBaudrate; 7] = [
    SerialBaudrate::B1200,
    SerialBaudrate::B2400,
    SerialBaudrate::B4800,
    SerialBaudrate::B9600,
    SerialBaudrate::B19200,
    SerialBaudrate::B38400,
    SerialBaudrate::B57600,
];

const DEFAULT_BAUD_RATE: SerialBaudrate = SerialBaudrate::B9600;

#[derive(Debug)]
pub struct IiyamaController {
    connection: ConnectionType,
    write_setting_requests: Vec<SetCommand>,
    read_setting_requests: Vec<GetCommand>,
}

impl IiyamaController {
    /// Create a new `IiyamaDisplayController` with a serial connection
    fn new_serial(
        settings: &SerialConnectionParameters,
    ) -> Result<Self, SerialPortConnectionError> {
        let mut compatible_settings = (*settings).clone();

        // Fallback to default baud rate if the provided one is not supported
        if !compatible_settings.is_valid_baud_rate(&SUPPORTED_BAUD_RATES) {
            eprintln!(
                "Unsupported baud rate. Falling back to default: {}",
                DEFAULT_BAUD_RATE as u32
            );

            compatible_settings.force(None, Some(DEFAULT_BAUD_RATE), None, None, None, None, None);
        }

        // Force 8 data bits, 1 stop bit, no parity, no flow control
        compatible_settings.force(
            None,
            None,
            Some(Some(serialport::DataBits::Eight)),
            Some(Some(serialport::StopBits::One)),
            Some(Some(serialport::Parity::None)),
            Some(Some(serialport::FlowControl::None)),
            None,
        );

        // Let's actually connect
        let port = compatible_settings.connect()?;

        Ok(Self {
            connection: ConnectionType::Serial(port),
            write_setting_requests: Vec::new(),
            read_setting_requests: Vec::new(),
        })
    }

    fn new_tcp(settings: &TcpConnectionParameters) -> std::io::Result<Self> {
        let mut compatible_settings: TcpConnectionParameters = (*settings).clone();

        // Force port 5000
        compatible_settings.force(None, Some(5000), None);

        // Connect to display
        let stream = compatible_settings.connect()?;

        Ok(Self {
            connection: ConnectionType::Tcp(stream),
            write_setting_requests: Vec::new(),
            read_setting_requests: Vec::new(),
        })
    }
}

impl DisplayController for IiyamaController {
    fn new_and_connect(
        connection_settings: UserConnectionSettings,
    ) -> Result<Self, ConnectionError> {
        match connection_settings {
            UserConnectionSettings::Serial {
                port,
                baud_rate,
                timeout,
            } => Self::new_serial(&SerialConnectionParameters::new(
                Some(port),
                baud_rate,
                None,
                None,
                None,
                None,
                timeout.map(std::time::Duration::from_millis), // Timeout default enforced by new function
            ))
            .map_err(ConnectionError::SerialConnectionError),

            UserConnectionSettings::Tcp { ip, port, timeout } => {
                Self::new_tcp(&TcpConnectionParameters::new(
                    ip,
                    port,
                    timeout.map(std::time::Duration::from_millis), // Timeout default enforced by new function
                ))
                .map_err(ConnectionError::TcpConnectionError)
            }
        }
    }

    fn add_write_setting_request(
        &mut self,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<(), AddSettingsError> {
        match name {
            "power.state" => match PowerState::deserialize(value) {
                Ok(state) => {
                    self.write_setting_requests
                        .push(SetCommand::PowerState(state));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama PowerState (see docs for details)",
                )),
            },
            "power.onstart" => match PowerStateAtColdStart::deserialize(value) {
                Ok(state) => {
                    self.write_setting_requests
                        .push(SetCommand::PowerStateAtColdStart(state));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama PowerStateAtColdStart (see docs for details)",
                )),
            },
            "input.source" => match InputSource::deserialize(value) {
                Ok(source) => {
                    self.write_setting_requests
                        .push(SetCommand::InputSource(source));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama InputSource (see docs for details)",
                )),
            },
            "hardware.ir.block" => match BlockedUserInput::deserialize(value) {
                Ok(block) => {
                    self.write_setting_requests
                        .push(SetCommand::IrRemoteControl(block));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama BlockedUserInput (see docs for details)",
                )),
            },
            "hardware.keypad.block" => match BlockedUserInput::deserialize(value) {
                Ok(block) => {
                    self.write_setting_requests
                        .push(SetCommand::KeypadControl(block));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama BlockedUserInput (see docs for details)",
                )),
            },
            "picture.format" => match PictureFormat::deserialize(value) {
                Ok(format) => {
                    self.write_setting_requests
                        .push(SetCommand::PictureFormat(format));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama PictureFormat (see docs for details)",
                )),
            },
            "picture.color.temperature" => match ColorTemperature::deserialize(value) {
                Ok(temp) => {
                    self.write_setting_requests
                        .push(SetCommand::ColorTemperature(temp));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama ColorTemperature (see docs for details)",
                )),
            },
            "picture.video.parameters" => match VideoParameters::deserialize(value) {
                Ok(mut params) => {
                    params.clip_values();
                    self.write_setting_requests
                        .push(SetCommand::VideoParameters(params));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama VideoParameters (see docs for details)",
                )),
            },
            "picture.color.parameters" => match ColorParameters::deserialize(value) {
                Ok(params) => {
                    self.write_setting_requests
                        .push(SetCommand::ColorParameters(params));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama ColorParameters (see docs for details)",
                )),
            },
            "audio.volume" => match Volume::deserialize(value) {
                Ok(mut volume) => {
                    volume.clip_values();
                    self.write_setting_requests.push(SetCommand::Volume(volume));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama Volume (see docs for details)",
                )),
            },
            "audio.volume.limits" => match VolumeLimits::deserialize(value) {
                Ok(mut limits) => {
                    limits.clip_values();
                    self.write_setting_requests
                        .push(SetCommand::VolumeLimits(limits));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama VolumeLimits (see docs for details)",
                )),
            },
            "audio.parameters" => match AudioParameters::deserialize(value) {
                Ok(mut params) => {
                    params.clip_values();
                    self.write_setting_requests
                        .push(SetCommand::AudioParameters(params));
                    Ok(())
                }
                Err(_) => Err(AddSettingsError::InvalidType(
                    "Iiyama AudioParameters (see docs for details)",
                )),
            },
            _ => Err(AddSettingsError::UnknownSetting),
        }
    }

    fn add_read_setting_request(&mut self, name: &str) -> Result<(), AddSettingsError> {
        todo!()
    }

    fn add_complete_read_settings_request(&mut self) -> Result<(), AddSettingsError> {
        todo!()
    }

    fn fetch_read_settings(&mut self) -> Result<SettingEntry, ExecuteSettingsError> {
        todo!()
    }

    fn apply_write_settings(
        &mut self,
        only_write_on_diff: bool,
    ) -> Result<Option<Vec<SettingDiff>>, ExecuteSettingsError> {
        todo!()
    }
}

trait IiyamaCommand {
    #[must_use]
    fn get_command_code(&self) -> u8;
    #[must_use]
    fn get_payload_data(&self) -> Option<Vec<u8>>;
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
    fn new_raw(monitor_id: u8, command_code: u8, data: &Option<Vec<u8>>) -> Self {
        let length = data.as_ref().map_or(0, |d| d.len() as u8) + 3;

        let data = vec![command_code]
            .into_iter()
            .chain(data.as_ref().map_or(vec![], std::clone::Clone::clone))
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

    #[must_use]
    fn new(monitor_id: u8, command: &dyn IiyamaCommand) -> Self {
        let function_code = command.get_command_code();
        let data = command.get_payload_data();

        Self::new_raw(monitor_id, function_code, &data)
    }
}

// TODO
fn set(monitor_id: u8, function: SetCommand, port: &mut Box<dyn SerialPort>) {
    let package = RequestPackage::new(monitor_id, &function);
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

enum RawResponseStatus<T> {
    Acknowledged(T),
    NotAcknowledged,
    NotAvailable,
}

type RawSetResponseStatus = RawResponseStatus<()>;
type RawGetResponseStatus<T> = RawResponseStatus<T>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the creation of a RawRequestPackage with simple data.
    #[test]
    fn test_raw_request_package() {
        let package = RequestPackage::new_raw(1, 0x19, &Some(vec![0x01]));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
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
        let package = RequestPackage::new_raw(1, 0x19, &None);
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 3);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data, Some(vec![0x19]));
        assert_eq!(package.checksum, 0xa6 ^ 1 ^ 3 ^ 1 ^ 0x19);
    }
}
