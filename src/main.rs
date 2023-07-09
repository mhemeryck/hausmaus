use std::thread::JoinHandle;
use std::thread::{sleep, spawn};
use std::time::Duration;

use clap::Parser;

use crossbeam::channel::bounded;
use crossbeam::channel::Receiver;
use crossbeam::select;
use hausmaus::models::{Cover, CoverEvent};

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

fn monitor(event_rx: Receiver<CoverEvent>, timer_rx: Receiver<CoverEvent>) -> JoinHandle<()> {
    spawn(move || {
        let mut cover = Cover::new(0, 1);
        loop {
            select! {
                recv(event_rx) -> msg => {
                    if let Ok(CoverEvent::PushButtonOpen) | Ok(CoverEvent::PushButtonClose) = msg {
                        log::info!("Got an event {:?}", msg);
                        cover.process_event(msg.unwrap());
                    }
                },
                recv(timer_rx) -> msg => {
                    log::info!("Got an timer {:?}", msg);
                },
            }
        }
    })
}

fn main() {
    let log_level = "info";
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
    //log::info!("New cover created {:?}", cover);

    let (event_tx, event_rx) = bounded(4);
    let (timer_tx, timer_rx) = bounded(4);

    let timer_handle = spawn(move || {
        let tick_duration = Duration::from_secs_f32(0.5);
        loop {
            log::info!("Sending tick");
            timer_tx.send(CoverEvent::TimerTick).unwrap();
            sleep(tick_duration);
        }
    });

    let monitor_handle = monitor(event_rx, timer_rx);

    log::info!("Sending open");
    event_tx.send(CoverEvent::PushButtonOpen).unwrap();
    sleep(Duration::from_secs(1));
    log::info!("Sending close");
    event_tx.send(CoverEvent::PushButtonClose).unwrap();
    sleep(Duration::from_secs(1));
    log::info!("Sending close");
    event_tx.send(CoverEvent::PushButtonClose).unwrap();
    sleep(Duration::from_secs(1));
    log::info!("Sending open");
    event_tx.send(CoverEvent::PushButtonOpen).unwrap();
    sleep(Duration::from_secs(1));

    timer_handle.join().unwrap();
    monitor_handle.join().unwrap();

    //let (tx, rx) = bounded(4);

    //cover.start(rx);

    //tx.send(CoverEvent::PushButtonOpen).unwrap();

    //std::thread::sleep(Duration::from_secs(5));

    //tx.send(CoverEvent::PushButtonOpen).unwrap();
    //tx.send(CoverEvent::PushButtonOpen).unwrap();
    //tx.send(CoverEvent::PushButtonClose).unwrap();
    //tx.send(CoverEvent::PushButtonOpen).unwrap();
    //tx.send(CoverEvent::PushButtonOpen).unwrap();
    //tx.send(CoverEvent::TimerTick).unwrap();
}
