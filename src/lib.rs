use std::process::exit;

use serde::Deserialize;

use crate::{
    connection::UserConnectionSettings,
    controller::{AddSettingsError, DisplayController, RequestedSetting},
};

pub mod connection;
pub mod controller;
pub mod dialect;

#[must_use]
pub fn get_screen_label_prefix(label: &Option<String>, screen_number: usize) -> String {
    label.as_ref().map_or_else(
        || format!("Screen {screen_number}: "),
        |l| format!("Screen \"{l}\": "),
    )
}

#[derive(Debug, Deserialize)]
pub struct ReadUserSettings {
    connection: UserConnectionSettings,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WriteUserSettings {
    dialect: String,
    connection: UserConnectionSettings,
    label: Option<String>,
    validate: bool,
    #[serde(rename = "settings")]
    requested_settings: Vec<RequestedSetting>,
}

pub fn apply_settings(settings: Vec<WriteUserSettings>) {
    if settings.is_empty() {
        eprintln!("No settings provided to apply.");
        exit(0);
    }

    let mut control_threads = Vec::new();

    for (screen_number, setting) in settings.into_iter().enumerate() {
        control_threads.push(std::thread::spawn(move || {
            // Select controller
            let controller = match setting.dialect.as_str() {
                "iiyama" => dialect::iiyama::IiyamaController::new_and_connect(setting.connection),
                _ => {
                    eprintln!(
                        "Screen \"{}\": Unsupported dialect: {}",
                        setting.label.as_ref().unwrap_or(&"unnamed".into()),
                        setting.dialect
                    );
                    return;
                }
            };

            // Check if connection was successful
            let mut controller = match controller {
                Ok(value) => value,
                Err(e) => {
                    eprintln!(
                        "{}Failed to connect to the display: {}",
                        get_screen_label_prefix(&setting.label, screen_number),
                        e
                    );
                    return;
                }
            };

            // Add settings to the controller queue
            for requested_setting in &setting.requested_settings {
                if let Err(e) = controller.add_write_setting_request(
                    requested_setting.get_name(),
                    requested_setting.get_value(),
                ) {
                    match e {
                        AddSettingsError::UnknownSetting => eprintln!(
                            "{}Unknown setting '{}'",
                            get_screen_label_prefix(&setting.label, screen_number),
                            requested_setting.get_name()
                        ),
                        AddSettingsError::InvalidValue(value) => eprintln!(
                            "{}Invalid value for setting '{}': {}",
                            get_screen_label_prefix(&setting.label, screen_number),
                            requested_setting.get_name(),
                            value
                        ),
                        AddSettingsError::InvalidType(expected_type) => eprintln!(
                            "{}Invalid type for setting '{}', expected: {}",
                            get_screen_label_prefix(&setting.label, screen_number),
                            requested_setting.get_name(),
                            expected_type
                        ),
                    }
                }
            }

            // Apply the settings
            if let Err(e) = controller.apply_write_settings(false) {
                eprintln!(
                    "{}Failed to apply settings: {}",
                    get_screen_label_prefix(&setting.label, screen_number),
                    e
                );
                return;
            }
        }));
    }

    for thread in control_threads {
        if let Err(e) = thread.join() {
            eprintln!("A control thread panicked: {e:?}");
        }
    }
}

fn get_settings(settings: Vec<ReadUserSettings>) {
    if settings.is_empty() {
        eprintln!("No settings provided to apply.");
        exit(0);
    }

    let mut control_threads = Vec::new();

    for (screen_number, setting) in settings.into_iter().enumerate() {
        control_threads.push(std::thread::spawn(move || {}));
    }

    for thread in control_threads {
        if let Err(e) = thread.join() {
            eprintln!("A control thread panicked: {e:?}");
        }
    }
}
