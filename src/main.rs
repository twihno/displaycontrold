use clap::Parser;
use display_serial_controller::iiyama;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
struct CliParams {
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    port: String,
    #[arg(short, long, default_value = "9600")]
    baud_rate: u32,
    #[arg(short, long, default_value = "0")]
    monitor_id: u8,
    #[arg(short, long)]
    display_type: String,
    #[arg(short, long)]
    command: String,
    #[arg(short, long)]
    value: Option<String>,
    #[arg(short, long)]
    software_serial_protocol: Option<bool>,
}

fn main() {
    let args = CliParams::parse();

    let mut port = serialport::new(&args.port, args.baud_rate)
        .open()
        .expect("Failed to open port");
    port.set_timeout(std::time::Duration::from_secs(1))
        .expect("Failed to set timeout");

    port.clear(serialport::ClearBuffer::All)
        .expect("Failed to clear port buffers");

    match args.display_type.as_str() {
        "iiyama" => {
            if let Ok(function) = iiyama::SetCommand::from_str(&args.command, &args.value.unwrap())
            {
                iiyama::set(args.monitor_id, function, &mut port);
            } else {
                eprintln!("Invalid command or value");
            }
        }
        _ => eprintln!("Unsupported display type: {}", args.display_type),
    }
}
