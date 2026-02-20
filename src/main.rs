use clap::{Parser, Subcommand};
use displaycontrold::{ReadSettings, WriteSettings};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: OperationMode,
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
/// Currently two operation modes are supported:
/// - `get`: Retrieve the current state/settings of all provided displays
/// - `set`: Apply the provided settings to the displays
enum OperationMode {
    /// Read the current display state/settings
    Get {
        #[arg(long)]
        config_file: Option<String>,
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Apply the provided settings to the displays
    Set {
        #[arg(long)]
        config_file: Option<String>,
        #[arg(short, long)]
        config: Option<String>,
    },
}

/// Retrieve the config: 1. file, 2. string
fn try_get_config(config_file: Option<String>, config_str: Option<String>) -> Option<String> {
    if let Some(file_path) = config_file {
        match std::fs::read_to_string(file_path) {
            Ok(content) => Some(content),
            Err(e) => {
                eprintln!("Failed to read the configuration file: {}", e);
                None
            }
        }
    } else if let Some(config_str) = config_str {
        Some(config_str)
    } else {
        eprintln!("No configuration provided. Use --config-file or --config.");
        None
    }
}

/// Actually try to parse the configuration from the provided string/file content
fn try_parse_config<T: DeserializeOwned>(config_str: &str) -> Option<T> {
    match serde_json::from_str::<T>(config_str) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!("Failed to parse the configuration: {}", e);
            None
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.mode {
        OperationMode::Get {
            config_file,
            config,
        } => {
            let Some(config_str) = try_get_config(config_file, config) else {
                return;
            };
            let Some(config) = try_parse_config::<Vec<ReadSettings>>(&config_str) else {
                return;
            };
            println!("{config:?}");
        }
        OperationMode::Set {
            config_file,
            config,
        } => {
            let Some(config_str) = try_get_config(config_file, config) else {
                return;
            };
            let Some(config) = try_parse_config::<Vec<WriteSettings>>(&config_str) else {
                return;
            };
        }
    }
}
