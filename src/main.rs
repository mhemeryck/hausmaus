use clap::Parser;

use hausmaus;
use hostname;
use slug;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(help = "MQTT broker host to connect to")]
    mqtt_host: String,

    // Optional sysfs root path to start scanning for files
    #[arg(long)]
    sysfs: Option<String>,

    // Optional host name to pass in, used for root MQTT topic
    #[arg(long)]
    device_name: Option<String>,

    // Optional arg to show debug information
    #[arg(long)]
    debug: bool,

    // Optional arg to set the MQTT client ID string. Defaults to `hausmaus`
    #[arg(long)]
    mqtt_client_id: Option<String>,
}

// device name from hostname
fn device_name() -> Option<String> {
    match hostname::get() {
        Ok(os_string) => match os_string.into_string() {
            Ok(str_ref) => Some(str_ref),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn main() {
    let cli = Cli::parse();

    let mut mqtt_host = cli.mqtt_host;
    if !mqtt_host.starts_with("tcp") {
        mqtt_host = format!("tcp://{}", mqtt_host);
    }

    let sysfs_path = match cli.sysfs.as_deref() {
        Some(sysfs_path) => sysfs_path,
        None => "/run/unipi",
    };

    let device_name: String = match cli.device_name.as_deref() {
        // from input arg
        Some(device_name) => device_name.to_string(),
        // from hostname
        None => device_name().unwrap(),
    };
    let device_name = slug::slugify(device_name);
    let device_name = device_name.as_str();

    let debug = cli.debug;

    let mqtt_client_id = match cli.mqtt_client_id.as_deref() {
        Some(mqtt_client_id) => mqtt_client_id,
        None => "hausmaus",
    };

    hausmaus::maus::run(&mqtt_host, &sysfs_path, &device_name, &mqtt_client_id, debug);
}
