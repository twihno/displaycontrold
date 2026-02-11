use thiserror::Error;

/// Common types and enums used across the application.

#[derive(Debug, Clone, Copy)]
pub enum PowerState {
    Off,
    On,
}

impl TryFrom<&str> for PowerState {
    type Error = FunctionConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "on" => Ok(PowerState::On),
            "off" => Ok(PowerState::Off),
            _ => Err(FunctionConversionError::InvalidValue(format!(
                "{value} doesn't match 'on' or 'off'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestResponse {
    Success,
    Error,
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FunctionConversionError {
    #[error("Unknown command")]
    UnknownCommand,
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}
