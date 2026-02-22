pub struct GetRequest {
    pub monitor_id: u8,
    pub command: GetCommand,
}

/// Enum representing the different request functions for getting data from the monitor
#[derive(Debug, Clone, Copy)]
pub enum GetCommand {
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
    pub fn get_command_code(&self) -> u8 {
        match self {
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
