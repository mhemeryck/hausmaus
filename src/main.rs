use std::thread::sleep;
use std::time::Duration;

use clap::Parser;

use crossbeam::channel::bounded;
use hausmaus::models::{Cover, CoverEvent, CoverPosition};

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

fn main2() {
    let cli = Cli::parse();

    let sysfs_path = cli.sysfs.as_deref().unwrap_or("/run/unipi");

    let device_name: String = match cli.device_name.as_deref() {
        // from input arg
        Some(device_name) => device_name.to_string(),
        // from hostname
        None => device_name().unwrap(),
    };
    let device_name = slug::slugify(device_name);
    let device_name = device_name.as_str();

    let debug = cli.debug;

    let mqtt_client_id = cli.mqtt_client_id.as_deref().unwrap_or("hausmaus");

    // log config
    let log_level = match debug {
        true => "debug",
        false => "info",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    hausmaus::maus::run(&cli.mqtt_host, sysfs_path, device_name, mqtt_client_id);
}

fn main() {
    let log_level = "debug";
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
    //log::info!("New cover created {:?}", cover);

    let cover = Cover::new(0, 1);

    // Event channel
    let (event_tx, event_rx) = bounded(4);
    let monitor_handle = cover.monitor(event_rx);

    log::info!("Sending close");
    event_tx.send(CoverEvent::PushButtonClose).unwrap();
    sleep(Duration::from_secs(10));
    log::info!("Sending open");
    event_tx.send(CoverEvent::PushButtonOpen).unwrap();

    monitor_handle.join().unwrap();
}
