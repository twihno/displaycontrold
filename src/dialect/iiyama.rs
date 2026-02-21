use serde::{Deserialize, Serialize};
use serialport::SerialPort;

use crate::{
    connection::{
        ConnectionError, ConnectionType, SerialBaudrate, SerialConnectionParameters,
        SerialPortConnectionError, TcpConnectionParameters, UserConnectionSettings,
    },
    controller::{
        AddSettingsError, DisplayController, ExecuteSettingsError, SettingDiff, SettingEntry,
    },
};

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
        validate: bool,
        get_diff: bool,
    ) -> Result<Option<Vec<SettingDiff>>, ExecuteSettingsError> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum PowerState {
    #[serde(rename = "off")]
    Off = 0x01,
    #[serde(rename = "on")]
    On = 0x02,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum BlockedUserInput {
    #[serde(rename = "none")]
    UnlockAll = 0x01,
    #[serde(rename = "all")]
    LockAll = 0x02,
    #[serde(rename = "except-power")]
    LockAllButPower = 0x03,
    #[serde(rename = "except-volume")]
    LockAllButVolume = 0x04,
    // Some weird daisy chaining. From doc:
    // > 0x05 = Primary (Master) - Reply OTS_SET_IR_ACK(0x21 0x01 0x00 0x00 0x04 0x01 0x00 0x00 0x25)
    // > 0x06 = Secondary (Daisy chain PD) - Reply OTS_SET_IR_ACK(0x21 0x01 0x00 0x00 0x04 0x01 0x00 0x00 0x25)
    // #[serde(rename = "master")]
    // Primary = 0x05,
    // #[serde(rename = "secondary")]
    // Secondary = 0x06,
    // ** but only for IR **
    #[serde(rename = "except-power-volume")]
    ExceptPowerVolume = 0x07,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PowerStateAtColdStart {
    #[serde(rename = "off")]
    PowerOff = 0x00,
    #[serde(rename = "on")]
    ForcedOn = 0x01,
    #[serde(rename = "last")]
    LastState = 0x02,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum InputSource {
    #[serde(rename = "video")]
    Video = 0x01,
    #[serde(rename = "s-video")]
    SVideo = 0x02,
    #[serde(rename = "component")]
    Component = 0x03,
    #[serde(rename = "cvi 2")]
    Cvi2 = 0x04,
    #[serde(rename = "vga")]
    Vga = 0x05,
    #[serde(rename = "hdmi 2")]
    HDMi2 = 0x06,
    #[serde(rename = "displayport 2")]
    DisplayPort2 = 0x07,
    #[serde(rename = "usb 2")]
    Usb2 = 0x08,
    #[serde(rename = "card dvi-d")]
    CardDviD = 0x09,
    #[serde(rename = "displayport 1")]
    DisplayPort1 = 0x0A,
    #[serde(rename = "card ops")]
    CardOps = 0x0B,
    #[serde(rename = "usb 1")]
    Usb1 = 0x0C,
    #[serde(rename = "hdmi")]
    HDMi = 0x0D,
    #[serde(rename = "dvi-d")]
    DviD = 0x0E,
    #[serde(rename = "hdmi 3")]
    HDMi3 = 0x0F,
    #[serde(rename = "browser")]
    Browser = 0x10,
    #[serde(rename = "smartcms")]
    SmartCMS = 0x11,
    #[serde(rename = "dms")]
    /// Digital Media Server (DMS)
    DMS = 0x12,
    #[serde(rename = "internal storage")]
    InternalStorage = 0x13,
    // 0x14 and 0x15 are reserved
    #[serde(rename = "media player")]
    MediaPlayer = 0x16,
    #[serde(rename = "pdf player")]
    PdfPlayer = 0x17,
    #[serde(rename = "custom")]
    Custom = 0x18,
    #[serde(rename = "hdmi 4")]
    HDMi4 = 0x19,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum GammaSelection {
    #[serde(rename = "native")]
    Native = 0x01,
    #[serde(rename = "s gamma")]
    SGamma = 0x02,
    #[serde(rename = "2.2")]
    Gamma22 = 0x03,
    #[serde(rename = "2.4")]
    Gamma24 = 0x04,
    #[serde(rename = "d-image")]
    /// D-Image (DICOM gamma)
    DImage = 0x05,
}

/// Need clipping ([`VideoParameters::clip_values`])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct VideoParameters {
    brightness: u8,
    color: u8,
    contrast: u8,
    sharpness: u8,
    /// Tint (Hue)
    tint: u8,
    black_level: u8,
    gamma: GammaSelection,
}

impl VideoParameters {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    fn clip_values(&mut self) {
        self.brightness = self.brightness.clamp(0, 100);
        self.color = self.color.clamp(0, 100);
        self.contrast = self.contrast.clamp(0, 100);
        self.sharpness = self.sharpness.clamp(0, 100);
        self.tint = self.tint.clamp(0, 100);
        self.black_level = self.black_level.clamp(0, 100);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum ColorTemperature {
    #[serde(rename = "user 1")]
    User1 = 0x00,
    #[serde(rename = "native")]
    Native = 0x01,
    #[serde(rename = "11000K")]
    /// Not applicable for some models
    K11000 = 0x02,
    #[serde(rename = "10000K")]
    K10000 = 0x03,
    #[serde(rename = "9300K")]
    K9300 = 0x04,
    #[serde(rename = "7500K")]
    K7500 = 0x05,
    #[serde(rename = "6500K")]
    K6500 = 0x06,
    #[serde(rename = "5770K")]
    /// Not applicable for some models
    K5770 = 0x07,
    #[serde(rename = "5500K")]
    /// Not applicable for some models
    K5500 = 0x08,
    #[serde(rename = "5000K")]
    K5000 = 0x09,
    #[serde(rename = "4000K")]
    K4000 = 0x0A,
    #[serde(rename = "3400K")]
    /// Not applicable for some models    
    K3400 = 0x0B,
    #[serde(rename = "3350K")]
    /// Not applicable for some models
    K3350 = 0x0C,
    #[serde(rename = "3000K")]
    K3000 = 0x0D,
    #[serde(rename = "2800K")]
    /// Not applicable for some models
    K2800 = 0x0E,
    #[serde(rename = "2600K")]
    /// Not applicable for some models
    K2600 = 0x0F,
    #[serde(rename = "1850K")]
    /// Not applicable for some models
    K1850 = 0x10,
    #[serde(rename = "user 2")]
    User2 = 0x12,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ColorParameters {
    red_gain: u8,
    green_gain: u8,
    blue_gain: u8,
    red_offset: u8,
    green_offset: u8,
    blue_offset: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PictureFormat {
    #[serde(rename = "4:3")]
    Normal43 = 0x00,
    #[serde(rename = "custom")]
    Custom = 0x01,
    #[serde(rename = "1:1")]
    Real11 = 0x02,
    #[serde(rename = "full")]
    Full = 0x03,
    #[serde(rename = "21:9")]
    Widescreen219 = 0x04,
    #[serde(rename = "dynamic")]
    Dynamic = 0x05,
    #[serde(rename = "16:9")]
    Widescreen169 = 0x06,
}

/// Need clipping ([`Volume::clip_values`])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Volume {
    volume: u8,
    #[serde(rename = "audio out level")]
    audio_out_volume_level: u8,
}

impl Volume {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    fn clip_values(&mut self) {
        self.volume = self.volume.clamp(0, 100);
        self.audio_out_volume_level = self.audio_out_volume_level.clamp(0, 100);
    }
}

/// Need clipping ([`VolumeLimits::clip_values`])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct VolumeLimits {
    #[serde(rename = "min")]
    min_volume: u8,
    #[serde(rename = "max")]
    max_volume: u8,
    #[serde(rename = "switch on")]
    switch_on_volume: u8,
}

impl VolumeLimits {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    fn clip_values(&mut self) {
        self.min_volume = self.min_volume.clamp(0, 100);
        self.max_volume = self.max_volume.clamp(0, 100);

        // I don't know if this is strictly necessary but I think it's at least logical
        self.switch_on_volume = self
            .switch_on_volume
            .clamp(self.min_volume, self.max_volume);
    }
}

/// Need clipping ([`AudioParameters::clip_values`])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct AudioParameters {
    treble: u8,
    bass: u8,
}

impl AudioParameters {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    fn clip_values(&mut self) {
        self.treble = self.treble.clamp(0, 100);
        self.bass = self.bass.clamp(0, 100);
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
}

/// Enum representing the different request functions for setting data on the monitor
#[derive(Debug, Clone)]
pub enum SetCommand {
    PowerState(PowerState),
    IrRemoteControl(BlockedUserInput),
    KeypadControl(BlockedUserInput),
    PowerStateAtColdStart(PowerStateAtColdStart),
    InputSource(InputSource),
    VideoParameters(VideoParameters),
    ColorTemperature(ColorTemperature),
    ColorParameters(ColorParameters),
    PictureFormat(PictureFormat),
    Volume(Volume),
    VolumeLimits(VolumeLimits),
    AudioParameters(AudioParameters),
    AutoAdjust, // From doc: "Command requests the display to make auto adjustment on VGA Input source" => No user selectable parameters
}

impl SetCommand {
    #[must_use]
    /// Returns the command code associated with the request function
    fn get_command_code(&self) -> u8 {
        match self {
            SetCommand::PowerState(_) => 0x18,
            SetCommand::IrRemoteControl(_) => 0x1c,
            SetCommand::KeypadControl(_) => 0x1a,
            SetCommand::PowerStateAtColdStart(_) => 0xa3,
            SetCommand::InputSource(_) => 0xac,
            SetCommand::VideoParameters(_) => 0x32,
            SetCommand::ColorTemperature(_) => 0x34,
            SetCommand::ColorParameters(_) => 0x36,
            SetCommand::PictureFormat(_) => 0x3a,
            SetCommand::Volume(_) => 0x44,
            SetCommand::VolumeLimits(_) => 0xb8,
            SetCommand::AudioParameters(_) => 0x42,
            SetCommand::AutoAdjust => 0x70,
        }
    }

    #[must_use]
    fn get_payload_data(&self) -> Option<Vec<u8>> {
        match self {
            &SetCommand::PowerState(value) => Some(vec![value as u8]),
            &SetCommand::IrRemoteControl(value) => Some(vec![value as u8]),
            &SetCommand::KeypadControl(value) => Some(vec![value as u8]),
            &SetCommand::PowerStateAtColdStart(value) => Some(vec![value as u8]),
            // data[1] - data[3] are reserved and must be 0
            &SetCommand::InputSource(value) => Some(vec![value as u8, 0, 0, 0]),
            &SetCommand::VideoParameters(params) => Some(vec![
                params.brightness,
                params.color,
                params.contrast,
                params.sharpness,
                params.tint,
                params.black_level,
                params.gamma as u8,
            ]),
            &SetCommand::ColorTemperature(value) => Some(vec![value as u8]),
            &SetCommand::ColorParameters(params) => Some(vec![
                params.red_gain,
                params.green_gain,
                params.blue_gain,
                params.red_offset,
                params.green_offset,
                params.blue_offset,
            ]),
            &SetCommand::PictureFormat(value) => Some(vec![value as u8]),
            &SetCommand::Volume(params) => Some(vec![params.volume, params.audio_out_volume_level]),
            &SetCommand::VolumeLimits(params) => Some(vec![
                params.min_volume,
                params.max_volume,
                params.switch_on_volume,
            ]),
            &SetCommand::AudioParameters(params) => Some(vec![params.treble, params.bass]),
            // 0x40: Auto Adjust (all other values are reserved); 0x00: Reserved, default 0
            SetCommand::AutoAdjust => Some(vec![0x40, 0x00]),
        }
    }

    fn get_matching_get_command(&self) -> Option<GetCommand> {
        match self {
            _ => None, // TODO
        }
    }
}

pub struct GetRequest {
    pub monitor_id: u8,
    pub command: GetCommand,
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
}

fn construct_set_data_package(monitor_id: u8, function: SetCommand) -> RequestPackage {
    let command_code = function.get_command_code();
    let data = function.get_payload_data();

    RequestPackage::new(monitor_id, command_code, &data)
}

// TODO
fn set(monitor_id: u8, function: SetCommand, port: &mut Box<dyn SerialPort>) {
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
        let package = RequestPackage::new(1, 0x19, &Some(vec![0x01]));
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
        let package = RequestPackage::new(1, 0x19, &None);
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

    #[test]
    fn test_set_power_state_off() {
        let package = construct_set_data_package(1, SetCommand::PowerState(PowerState::Off));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
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
        let package = construct_set_data_package(1, SetCommand::PowerState(PowerState::On));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x18, 0x02]);
        assert_eq!(package.checksum, 0xb8);
    }

    #[test]
    fn test_unlock_remote_control() {
        let package =
            construct_set_data_package(1, SetCommand::IrRemoteControl(BlockedUserInput::UnlockAll));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x1c, 0x01]);
        assert_eq!(package.checksum, 0xbf);
    }

    #[test]
    fn test_unlock_keypad() {
        let package =
            construct_set_data_package(1, SetCommand::KeypadControl(BlockedUserInput::UnlockAll));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x1a, 0x01]);
        assert_eq!(package.checksum, 0xb9);
    }

    #[test]
    fn test_power_cold_start_last_state() {
        let package = construct_set_data_package(
            1,
            SetCommand::PowerStateAtColdStart(PowerStateAtColdStart::LastState),
        );
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0xa3, 0x02]);
        assert_eq!(package.checksum, 0x03);
    }

    #[test]
    fn test_input_source_dvi_d() {
        let package = construct_set_data_package(1, SetCommand::InputSource(InputSource::DviD));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x07);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0xac, 0x0e, 0x00, 0x00, 0x00]);
        assert_eq!(package.checksum, 0x03);
    }

    #[test]
    fn test_set_all_video_parameters_to_55() {
        let params = VideoParameters {
            brightness: 55,
            color: 55,
            contrast: 55,
            sharpness: 55,
            tint: 55,
            black_level: 55,
            gamma: GammaSelection::Gamma22,
        };
        let package = construct_set_data_package(1, SetCommand::VideoParameters(params));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x0a);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(
            package.data.unwrap(),
            vec![0x32, 0x37, 0x37, 0x37, 0x37, 0x37, 0x37, 0x03]
        );
        assert_eq!(package.checksum, 0x9d); // TODO: Verify this checksum calculation is correct
    }

    #[test]
    fn test_set_color_temperature_native() {
        let package =
            construct_set_data_package(1, SetCommand::ColorTemperature(ColorTemperature::Native));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x34, 0x01]);
        assert_eq!(package.checksum, 0x97);
    }

    #[test]
    fn test_set_all_color_parameters_to_255() {
        let params = ColorParameters {
            red_gain: 255,
            green_gain: 255,
            blue_gain: 255,
            red_offset: 255,
            green_offset: 255,
            blue_offset: 255,
        };
        let package = construct_set_data_package(1, SetCommand::ColorParameters(params));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x09);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(
            package.data.unwrap(),
            vec![0x36, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(package.checksum, 0x99);
    }

    #[test]
    fn test_set_format_full() {
        let package = construct_set_data_package(1, SetCommand::PictureFormat(PictureFormat::Full));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x04);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x3a, 0x03]);
        assert_eq!(package.checksum, 0x9b);
    }

    #[test]
    fn test_set_volume_level_77() {
        let volume = Volume {
            volume: 77,
            audio_out_volume_level: 77,
        };
        let package = construct_set_data_package(1, SetCommand::Volume(volume));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x05);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x44, 0x4d, 0x4d]);
        assert_eq!(package.checksum, 0xe7);
    }

    #[test]
    fn test_set_volume_limits() {
        let limits = VolumeLimits {
            min_volume: 10,
            max_volume: 77,
            switch_on_volume: 50,
        };
        let package = construct_set_data_package(1, SetCommand::VolumeLimits(limits));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x06);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0xb8, 0x0a, 0x4d, 0x32]);
        assert_eq!(package.checksum, 0x6d);
    }

    #[test]
    fn test_set_audio_parameters_77() {
        let params = AudioParameters {
            treble: 77,
            bass: 77,
        };
        let package = construct_set_data_package(1, SetCommand::AudioParameters(params));
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x05);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x42, 0x4d, 0x4d]);
        assert_eq!(package.checksum, 0xe1);
    }

    #[test]
    fn test_set_auto_adjust() {
        let package = construct_set_data_package(1, SetCommand::AutoAdjust);
        assert_eq!(package.header, 0xa6);
        assert_eq!(package.monitor_id, 0x01);
        assert_eq!(package.category, 0x00);
        assert_eq!(package.code0, 0x00);
        assert_eq!(package.code1, 0x00);
        assert_eq!(package.length, 0x05);
        assert_eq!(package.data_control, 0x01);
        assert_eq!(package.data.unwrap(), vec![0x70, 0x40, 0x00]);
        assert_eq!(package.checksum, 0x93);
    }
}
