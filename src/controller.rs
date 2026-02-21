use serde::Deserialize;
use thiserror::Error;

use crate::connection::{ConnectionError, UserConnectionSettings};

#[derive(Debug, Deserialize)]
pub struct TimeFilter {
    start: String,
    end: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestedSetting {
    name: String,
    value: serde_json::Value,
    time_filter: Option<TimeFilter>,
}

impl RequestedSetting {
    #[must_use] 
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[must_use] 
    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SettingEntry {
    name: String,
    value: serde_json::Value,
}

impl SettingEntry {
    #[must_use]
    pub fn new(name: String, value: serde_json::Value) -> Self {
        Self { name, value }
    }

    #[must_use] 
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[must_use] 
    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValueDiff<T> {
    old_value: T,
    new_value: T,
}

#[derive(Debug, Clone)]
pub enum SettingValueDiff {
    Unchanged,
    Changed(ValueDiff<serde_json::Value>),
}

#[derive(Debug, Clone)]
pub struct SettingDiff {
    name: String,
    value_diff: SettingValueDiff,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddSettingsError {
    #[error("Unknown command")]
    UnknownSetting,
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Invalid type for value, expected: {0}")]
    InvalidType(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExecuteSettingsError {
    #[error("Communication error: {0}")]
    CommunicationError(std::io::ErrorKind),
    #[error("Setting unavailable: {0}")]
    SettingUnavailable(String),
    #[error("Failed setting: {0}")]
    FailedSetting(serde_json::Value),
}

/// The common interface for all manufacturer-specific implementations for the communication with the display.
pub trait DisplayController {
    #[must_use]
    fn new_and_connect(
        connection_settings: UserConnectionSettings,
    ) -> Result<Self, ConnectionError>
    where
        Self: Sized;

    /// Try to parse the requested setting.
    /// E.g also check if the setting exists for this manufactured (not yet for this model) and if the value is of the correct type, ...
    fn add_write_setting_request(
        &mut self,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<(), AddSettingsError>;

    /// Add a single setting to the read queue, so that it can be fetched with [`Self::fetch_read_settings`]
    fn add_read_setting_request(&mut self, name: &str) -> Result<(), AddSettingsError>;

    /// Add all known settings for this display to the read queue, so that they can be fetched with [`Self::fetch_read_settings`]
    fn add_complete_read_settings_request(&mut self) -> Result<(), AddSettingsError>;

    fn fetch_read_settings(&mut self) -> Result<SettingEntry, ExecuteSettingsError>;

    fn apply_write_settings(
        &mut self,
        validate: bool,
        get_diff: bool,
    ) -> Result<Option<Vec<SettingDiff>>, ExecuteSettingsError>;
    //  {
    //     let mut diffs = Vec::new();

    //     for requested_setting in requested_settings {
    //         let diff = self.set_single_setting(
    //             &requested_setting.name,
    //             &requested_setting.value,
    //             get_diff,
    //         )?;
    //         if let Some(diff) = diff {
    //             diffs.push(diff);
    //         }

    //         if validate {
    //             let current_setting = self.add_read_setting_request(&requested_setting.name)?;

    //             if current_setting.get_value() != requested_setting.get_value() {
    //                 return Err(SettingsError::ValidationError(format!(
    //                     "Validation failed for setting '{}': expected value {:?}, got {:?}",
    //                     requested_setting.name,
    //                     requested_setting.value,
    //                     current_setting.get_value()
    //                 )));
    //             }
    //         }
    //     }

    //     if get_diff { Ok(Some(diffs)) } else { Ok(None) }
    // }
}
