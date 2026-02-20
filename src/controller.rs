use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UntypedSettingEntry {
    name: String,
    value: String,
}

impl UntypedSettingEntry {
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

/// A single value of a display setting
#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingEntry {
    name: String,
    value: SettingValue,
    immutable: bool,
}

impl SettingEntry {
    #[must_use]
    pub fn new(name: String, value: SettingValue, immutable: bool) -> Self {
        Self {
            name,
            value,
            immutable,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &SettingValue {
        &self.value
    }

    pub fn try_set_value(&mut self, new_value: SettingValue) {
        if !self.immutable {
            self.value = new_value;
        }
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
    Changed(ValueDiff<SettingValue>),
}

#[derive(Debug, Clone)]
pub struct SettingDiff {
    name: String,
    value_diff: SettingValueDiff,
}

/// The common interface for all manufacturer-specific implementations for the communication with the display.
pub trait DisplayController {
    /// Retrieves the current settings of the display.
    fn get_settings(&self) -> Result<HashMap<String, SettingEntry>, String>; // TODO: Change return type 

    /// Sets a single setting of the display and returns the difference between the old and new value.
    fn set_single_setting(&mut self, name: &str, value: String) -> Result<SettingDiff, String>; // TODO: Change return type

    fn apply_settings(&mut self, settings: HashMap<String, SettingEntry>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_setting_entry() {
        let mut entry =
            SettingEntry::new("brightness".to_string(), SettingValue::Integer(50), false);
        assert_eq!(entry.get_name(), "brightness");
        assert_eq!(entry.get_value(), &SettingValue::Integer(50));

        entry.try_set_value(SettingValue::Integer(70));
        assert_eq!(entry.get_value(), &SettingValue::Integer(70));
    }

    #[test]
    fn test_immutable_setting_entry() {
        let mut immutable_entry = SettingEntry::new(
            "serial_number".to_string(),
            SettingValue::String("12345".to_string()),
            true,
        );
        assert_eq!(immutable_entry.get_name(), "serial_number");
        assert_eq!(
            immutable_entry.get_value(),
            &SettingValue::String("12345".to_string())
        );

        immutable_entry.try_set_value(SettingValue::String("54321".to_string()));
        assert_eq!(
            immutable_entry.get_value(),
            &SettingValue::String("12345".to_string())
        );
    }
}
