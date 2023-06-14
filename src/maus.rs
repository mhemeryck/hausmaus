use env_logger;
use futures;
use log;
use rumqttc;
use std;

const MQTT_KEEP_ALIVE: u64 = 20;
const MQTT_CLIENT_CHANNEL_CAP: usize = 10;

/// run is the main entry point to start the maus
///
/// It spawns:
/// - all input reader threads
/// - all output write threads
/// - the main automation engine thread to link input events to output events
#[tokio::main]
pub async fn run(
    mqtt_host: &str,
    sysfs_path: &str,
    device_name: &str,
    mqtt_client_id: &str,
    debug: bool,
) {
    // log config
    let log_level = match debug {
        true => "debug",
        false => "info",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    log::debug!("Start hausmaus");

    // Crawl a folder for paths to watch based on a regex
    log::debug!("Start crawling path {:?}", sysfs_path);
    let mut devices: std::vec::Vec<crate::device::Device> = std::vec::Vec::new();
    crate::device::devices_from_path(sysfs_path, device_name, &mut devices).unwrap();
    log::info!("Finished crawling path {:?}", sysfs_path);
    for device in &devices {
        log::debug!("Found device with id {} {:?}", device.id, device.path);
    }
    log::debug!("Number of devices: {}", devices.len());

    log::debug!("Build mapping of state topics for devices");
    let mut state_topic_map: std::collections::HashMap<u8, String> =
        std::collections::HashMap::new();
    crate::device::device_state_topics(&devices, &mut state_topic_map);

    log::debug!("Build mapping of command topics for devices");
    let mut command_topic_map: std::collections::HashMap<String, u8> =
        std::collections::HashMap::new();
    crate::device::device_command_topics(&devices, &mut command_topic_map);

    log::debug!("Build mapping of paths for devices");
    let mut path_map: std::collections::HashMap<u8, String> = std::collections::HashMap::new();
    crate::device::device_paths(&devices, &mut path_map);

    // MQTT setup
    let mut mqtt_options = rumqttc::MqttOptions::new(mqtt_client_id, mqtt_host, 1883);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(MQTT_KEEP_ALIVE));

    let (mqtt_client, mut mqtt_loop) =
        rumqttc::AsyncClient::new(mqtt_options, MQTT_CLIENT_CHANNEL_CAP);
    let mqtt_client = std::sync::Arc::new(mqtt_client);

    // Subscribe
    crate::mqtt::subscribe::subscribe_topics(&mqtt_client, &command_topic_map).await;

    // Channels
    let (file_read_tx, file_read_rx) = std::sync::mpsc::channel();
    let (mqtt_publish_tx, mqtt_publish_rx) = std::sync::mpsc::channel();
    let (log_write_tx, log_write_rx) = std::sync::mpsc::channel();
    let (mqtt_subscribe_tx, mqtt_subscribe_rx) = std::sync::mpsc::channel();
    let (file_write_tx, file_write_rx) = std::sync::mpsc::channel();

    let mut handles = std::vec::Vec::new();

    log::debug!("Start main file event watcher thread");
    let tx = file_read_tx.clone();
    let handle = tokio::spawn(async move {
        crate::sysfs::read::watch_input_file_events(devices.clone(), tx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to write to events");
    let handle = tokio::spawn(async move {
        crate::dummy::write_events(log_write_rx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to connect path sysfs read -> mqtt publish");
    let handle = tokio::spawn(async move {
        crate::auto::run_sysfs_to_mqtt(file_read_rx, log_write_tx, mqtt_publish_tx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to connect to handle MQTT publishing");
    let c = mqtt_client.clone();
    let handle = tokio::spawn(async move {
        crate::mqtt::publish::publish_messages(mqtt_publish_rx, &c, &state_topic_map).await
    });
    handles.push(handle);

    log::debug!("Start thread to connect MQTT subscribe -> sys write");
    let handle = tokio::spawn(async move {
        crate::auto::run_mqtt_to_sysfs(mqtt_subscribe_rx, file_write_tx).await;
    });
    handles.push(handle);

    log::debug!("Start thread to subscribe to and handle MQTT command topics");
    let handle = tokio::spawn(async move {
        crate::mqtt::subscribe::handle_incoming_messages(mqtt_subscribe_tx, &mut mqtt_loop).await
    });
    handles.push(handle);

    log::debug!("Start thread to subscribe to and handle MQTT command topics");
    let handle = tokio::spawn(async move {
        crate::sysfs::write::handle_file_command(file_write_rx, &path_map).await;
    });
    handles.push(handle);

    // Block on the handles processing
    futures::future::join_all(handles).await;
}
