use env_logger;
use log;
use paho_mqtt;
use regex;
use std;
use futures;

// Check whether we need all devices here or just the digital inputs
const FILENAME_PATTERN: &str =
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";
const MQTT_HOST: &str = "tcp://emqx.mhemeryck.com";
const MQTT_CLIENT_ID: &str = "hausmaus";
const MQTT_KEEP_ALIVE: u64 = 20;

/// run is the main entry point to start the maus
///
/// It spawns:
/// - all input reader threads
/// - all output write threads
/// - the main automation engine thread to link input events to output events
#[tokio::main]
pub async fn run(sysfs_path: &str, device_name: &str, debug: bool) {
    // log config
    let log_level = match debug {
        true => "debug",
        false => "info",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    log::debug!("Start hausmaus");
    // Crawl a folder for paths to watch based on a regex
    log::debug!("Start crawling path {:?}", sysfs_path);
    let mut paths: std::vec::Vec<std::path::PathBuf> = std::vec::Vec::new();
    let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
    crate::sysfs::crawl(&std::path::Path::new(&sysfs_path), &re, &mut paths).unwrap();
    log::debug!("Finished crawling path {:?}", sysfs_path);

    // turn into mut ref
    let device_name = device_name.to_string();

    let mut handles = std::vec::Vec::new();

    // create list of devices
    let mut devices: std::vec::Vec<crate::device::Device> = std::vec::Vec::new();
    crate::sysfs::devices_from_paths(device_name.as_str(), &paths, &mut devices);

    // file read channel
    let (file_read_tx, file_read_rx) = std::sync::mpsc::channel();

    // MQTT setup
    let create_opts = paho_mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_HOST)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();
    let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(MQTT_KEEP_ALIVE))
        .clean_session(true)
        .finalize();
    let mqtt_client = std::sync::Arc::new(paho_mqtt::AsyncClient::new(create_opts).unwrap());
    mqtt_client.connect(conn_opts).await.unwrap();
    let (mqtt_publish_tx, mqtt_publish_rx) = std::sync::mpsc::channel();

    // dummy log write channel
    let (log_write_tx, log_write_rx) = std::sync::mpsc::channel();

    log::debug!("Start main file event watcher thread");
    let file_event_paths = paths.clone();
    let file_event_tx = file_read_tx.clone();
    let device_name_clone = device_name.clone();
    let handle = tokio::spawn(async move {
        crate::sysfs::read::watch_input_file_events(
            file_event_paths,
            device_name_clone,
            file_event_tx,
        )
        .await;
    });
    handles.push(handle);

    log::debug!("Start thread to write to events");
    let handle = tokio::spawn(async move {
        crate::dummy::write_events(log_write_rx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to connect all together");
    let handle = tokio::spawn(async move {
        crate::auto::run_sysfs_to_mqtt(file_read_rx, log_write_tx, mqtt_publish_tx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to connect to handle MQTT publishing");
    let mqtt_publish_client = mqtt_client.clone();
    let handle = tokio::spawn(async move {
        crate::mqtt::publish::publish_messages(mqtt_publish_rx, &mqtt_publish_client)
            .await
            .unwrap();
    });
    handles.push(handle);

    let (mqtt_subscribe_tx, mqtt_subscribe_rx): (
        std::sync::mpsc::Sender<paho_mqtt::Message>,
        std::sync::mpsc::Receiver<paho_mqtt::Message>,
    ) = std::sync::mpsc::channel();

    let handle = tokio::spawn(async move {
        crate::auto::run_mqtt_to_sysfs(mqtt_subscribe_rx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to subscribe to MQTT command topics");
    let mqtt_subscribe_tx_clone = mqtt_subscribe_tx.clone();
    let handle = tokio::spawn(async move {
        crate::mqtt::subscribe::handle_incoming_messages(
            &mqtt_client,
            &devices,
            mqtt_subscribe_tx_clone,
        )
        .await
        .unwrap();
    });
    handles.push(handle);

    // Block on the handles processing
    futures::future::join_all(handles).await;
}
