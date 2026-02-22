use serde::{Deserialize, Serialize};

use crate::dialect::iiyama::{IiyamaCommand, get_commands::GetCommand};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PowerState {
    #[serde(rename = "off")]
    Off = 0x01,
    #[serde(rename = "on")]
    On = 0x02,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockedUserInput {
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
pub enum PowerStateAtColdStart {
    #[serde(rename = "off")]
    PowerOff = 0x00,
    #[serde(rename = "on")]
    ForcedOn = 0x01,
    #[serde(rename = "last")]
    LastState = 0x02,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputSource {
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
pub enum GammaSelection {
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
pub struct VideoParameters {
    pub brightness: u8,
    pub color: u8,
    pub contrast: u8,
    pub sharpness: u8,
    /// Tint (Hue)
    pub tint: u8,
    pub black_level: u8,
    pub gamma: GammaSelection,
}

impl VideoParameters {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    pub fn clip_values(&mut self) {
        self.brightness = self.brightness.clamp(0, 100);
        self.color = self.color.clamp(0, 100);
        self.contrast = self.contrast.clamp(0, 100);
        self.sharpness = self.sharpness.clamp(0, 100);
        self.tint = self.tint.clamp(0, 100);
        self.black_level = self.black_level.clamp(0, 100);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorTemperature {
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
pub struct ColorParameters {
    pub red_gain: u8,
    pub green_gain: u8,
    pub blue_gain: u8,
    pub red_offset: u8,
    pub green_offset: u8,
    pub blue_offset: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PictureFormat {
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
pub struct Volume {
    pub volume: u8,
    #[serde(rename = "audio out level")]
    pub audio_out_volume_level: u8,
}

impl Volume {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    pub fn clip_values(&mut self) {
        self.volume = self.volume.clamp(0, 100);
        self.audio_out_volume_level = self.audio_out_volume_level.clamp(0, 100);
    }
}

/// Need clipping ([`VolumeLimits::clip_values`])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeLimits {
    #[serde(rename = "min")]
    pub min_volume: u8,
    #[serde(rename = "max")]
    pub max_volume: u8,
    #[serde(rename = "switch on")]
    pub switch_on_volume: u8,
}

impl VolumeLimits {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    pub fn clip_values(&mut self) {
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
pub struct AudioParameters {
    pub treble: u8,
    pub bass: u8,
}

impl AudioParameters {
    /// Clip the (potentially deserialized) values to the allowed range of 0-100
    /// This **must be called** before sending the data to the screen
    pub fn clip_values(&mut self) {
        self.treble = self.treble.clamp(0, 100);
        self.bass = self.bass.clamp(0, 100);
    }
}

/// Enum representing the different request functions for setting data on the monitor
#[derive(Debug, Clone)]
#[repr(u8)]
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

impl IiyamaCommand for SetCommand {
    fn get_command_code(&self) -> u8 {
        match self {
            Self::PowerState(_) => 0x18,
            Self::IrRemoteControl(_) => 0x1c,
            Self::KeypadControl(_) => 0x1a,
            Self::PowerStateAtColdStart(_) => 0xa3,
            Self::InputSource(_) => 0xac,
            Self::VideoParameters(_) => 0x32,
            Self::ColorTemperature(_) => 0x34,
            Self::ColorParameters(_) => 0x36,
            Self::PictureFormat(_) => 0x3a,
            Self::Volume(_) => 0x44,
            Self::VolumeLimits(_) => 0xb8,
            Self::AudioParameters(_) => 0x42,
            Self::AutoAdjust => 0x70,
        }
    }

    fn get_payload_data(&self) -> Option<Vec<u8>> {
        match self {
            &Self::PowerState(value) => Some(vec![value as u8]),
            &Self::IrRemoteControl(value) => Some(vec![value as u8]),
            &Self::KeypadControl(value) => Some(vec![value as u8]),
            &Self::PowerStateAtColdStart(value) => Some(vec![value as u8]),
            // data[1] - data[3] are reserved and must be 0
            &Self::InputSource(value) => Some(vec![value as u8, 0, 0, 0]),
            &Self::VideoParameters(params) => Some(vec![
                params.brightness,
                params.color,
                params.contrast,
                params.sharpness,
                params.tint,
                params.black_level,
                params.gamma as u8,
            ]),
            &Self::ColorTemperature(value) => Some(vec![value as u8]),
            &Self::ColorParameters(params) => Some(vec![
                params.red_gain,
                params.green_gain,
                params.blue_gain,
                params.red_offset,
                params.green_offset,
                params.blue_offset,
            ]),
            &Self::PictureFormat(value) => Some(vec![value as u8]),
            &Self::Volume(params) => Some(vec![params.volume, params.audio_out_volume_level]),
            &Self::VolumeLimits(params) => Some(vec![
                params.min_volume,
                params.max_volume,
                params.switch_on_volume,
            ]),
            &Self::AudioParameters(params) => Some(vec![params.treble, params.bass]),
            // 0x40: Auto Adjust (all other values are reserved); 0x00: Reserved, default 0
            Self::AutoAdjust => Some(vec![0x40, 0x00]),
        }
    }
}

impl SetCommand {
    fn get_matching_get_command(&self) -> Option<GetCommand> {
        match self {
            Self::PowerState(_) => None,
            Self::IrRemoteControl(_) => None,
            Self::KeypadControl(_) => None,
            Self::PowerStateAtColdStart(_) => None,
            Self::InputSource(_) => None,
            Self::VideoParameters(_) => None,
            Self::ColorTemperature(_) => None,
            Self::ColorParameters(_) => None,
            Self::PictureFormat(_) => None,
            Self::Volume(_) => None,
            Self::VolumeLimits(_) => None,
            Self::AudioParameters(_) => None,
            Self::AutoAdjust => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dialect::iiyama::RequestPackage;

    use super::*;

    #[test]
    fn test_set_power_state_off() {
        let package = RequestPackage::new(1, &SetCommand::PowerState(PowerState::Off));
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
        let package = RequestPackage::new(1, &SetCommand::PowerState(PowerState::On));
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
            RequestPackage::new(1, &SetCommand::IrRemoteControl(BlockedUserInput::UnlockAll));
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
            RequestPackage::new(1, &SetCommand::KeypadControl(BlockedUserInput::UnlockAll));
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
        let package = RequestPackage::new(
            1,
            &SetCommand::PowerStateAtColdStart(PowerStateAtColdStart::LastState),
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
        let package = RequestPackage::new(1, &SetCommand::InputSource(InputSource::DviD));
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
        let package = RequestPackage::new(1, &SetCommand::VideoParameters(params));
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
            RequestPackage::new(1, &SetCommand::ColorTemperature(ColorTemperature::Native));
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
        let package = RequestPackage::new(1, &SetCommand::ColorParameters(params));
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
        let package = RequestPackage::new(1, &SetCommand::PictureFormat(PictureFormat::Full));
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
        let package = RequestPackage::new(1, &SetCommand::Volume(volume));
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
        let package = RequestPackage::new(1, &SetCommand::VolumeLimits(limits));
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
        let package = RequestPackage::new(1, &SetCommand::AudioParameters(params));
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
        let package = RequestPackage::new(1, &SetCommand::AutoAdjust);
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
