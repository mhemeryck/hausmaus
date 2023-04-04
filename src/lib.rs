use notify::{self, Watcher};
use paho_mqtt as mqtt;
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::sync;
use std::thread;
use std::time;
use std::vec;

const FILENAME: &str = "ro_value";
const SUB_FILENAME: &str = "/home/mhemeryck/Projects/unipinotifiy/fixtures/sys/devices/platform/unipi_plc/io_group2/ro_2_03/ro_value";
const POLL_INTERVAL: u64 = 200;

const MQTT_HOST: &str = "tcp://emqx.mhemeryck.com";
const MQTT_CLIENT_ID: &str = "hausmaus";
const MQTT_KEEP_ALIVE: u64 = 20;

const MQTT_TOPIC: &str = "foo";
// const MQTT_PAYLOAD: &[u8; 6] = b"Hello!";
const MQTT_QOS: i32 = 2;
const MQTT_TOPICS: &[&str] = &["bar", "qux"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::thread;
    use tempdir;

    #[test]
    fn test_crawl_simple_file_no_match() {
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        let path = tmp_dir.path().join("myfile.txt");
        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();

        crawl(tmp_dir.path(), &mut paths, "foo").expect("Expect crawl to work");

        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_crawl_file_match() {
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        let path = tmp_dir.path().join("myfile.txt");
        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();

        crawl(tmp_dir.path(), &mut paths, "myfile.txt").expect("Expect crawl to work");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], path);
    }

    #[test]
    fn test_match_event() {
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        let path = tmp_dir.path().join(FILENAME);
        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
        write!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let (tx, rx) = sync::mpsc::channel();

        // PollWatcher setup
        let config = notify::Config::default()
            .with_poll_interval(time::Duration::from_millis(POLL_INTERVAL))
            .with_compare_contents(true);
        let mut watcher = notify::PollWatcher::new(tx, config).unwrap();

        setup_watcher(
            &tmp_dir
                .path()
                .to_str()
                .expect("Could not convert temp folder path to string"),
            &mut watcher,
        )
        .expect("Could not set up watcher");

        // Writer thread: puts message on the sender
        let handle = thread::spawn(move || {
            write!(tmp_file, " world").expect("Could not write contents to temp file");
        });

        handle.join().expect("Writer did not complete!");
        let contents = fs::read_to_string(path).expect("Could not open temp file for reading");
        assert_eq!(contents, "Hello world");

        // Blocking wait to retrieve event
        let retrieved = rx.recv().expect("Error retrieving event");
        let event = retrieved.expect("Could not unwrap event");
        for path in event.paths {
            let contents =
                fs::read_to_string(path.to_str().expect("Could not convert path to string"))
                    .expect("Could not open temp file for reading");
            assert_eq!(contents, "Hello world");
        }
    }

    #[test]
    fn test_device_path() {
        let device = Device {
            device_type: DeviceType::DigitalInput,
            io_group: 1,
            number: 3,
        };
        assert_eq!(
            device.path("/var/run"),
            "/var/run/io_group_1/di_1_03/di_value"
        );
        let device = Device {
            device_type: DeviceType::DigitalOutput,
            io_group: 2,
            number: 4,
        };
        assert_eq!(
            device.path("/var/run"),
            "/var/run/io_group_2/do_2_04/do_value"
        );
        let device = Device {
            device_type: DeviceType::RelayOutput,
            io_group: 3,
            number: 11,
        };
        assert_eq!(
            device.path("/var/run"),
            "/var/run/io_group_3/ro_3_11/ro_value"
        );
    }
}

/// Crawls a directory structure for filenames matching given input
fn crawl(dir: &path::Path, paths: &mut vec::Vec<path::PathBuf>, filename: &str) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry: fs::DirEntry = entry?;
            let path = entry.path();
            if path.is_dir() {
                crawl(&path, paths, filename)?;
            } else if !path.is_symlink() {
                match path.to_str() {
                    Some(path_str) => {
                        if path_str.contains(filename) {
                            paths.push(path);
                        }
                    }
                    None => {}
                }
            }
        }
    }
    Ok(())
}

/// setup_watcher configures the file watcher to search for specific paths
fn setup_watcher(path_str: &str, watcher: &mut notify::PollWatcher) -> Result<(), notify::Error> {
    // Paths
    let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
    crawl(&path::Path::new(&path_str), &mut paths, FILENAME)?;

    for path in paths.iter() {
        // println!("Path {:?}", path.canonicalize().unwrap());
        watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;
    }

    Ok(())
}

/// handle_messages receives any file events and sends them out over MQTT
fn handle_messages(
    rx: &sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    mqtt_client: &mqtt::Client,
) -> paho_mqtt::errors::Result<()> {
    for res in rx {
        if let Ok(event) = res {
            println!("changed: {:?}", event);
            println!("event kind: {:?}", event.kind);
            for path in event.paths {
                if let Some(path_str) = path.to_str() {
                    let message = mqtt::Message::new(MQTT_TOPIC, path_str.as_bytes(), MQTT_QOS);
                    mqtt_client.publish(message)?;
                }
            }
        }
    }
    Ok(())
}

fn subscribe_topics(client: &mqtt::Client) -> paho_mqtt::errors::Result<()> {
    for topic in MQTT_TOPICS {
        client.subscribe(topic, MQTT_QOS)?;
    }
    Ok(())
}

pub fn run(path_str: &str) {
    let (tx, rx): (
        sync::mpsc::Sender<Result<notify::Event, notify::Error>>,
        sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) = sync::mpsc::channel();

    // PollWatcher setup
    let config = notify::Config::default()
        .with_poll_interval(time::Duration::from_millis(POLL_INTERVAL))
        .with_compare_contents(true);
    let mut watcher = notify::PollWatcher::new(tx, config).unwrap();

    setup_watcher(&path_str, &mut watcher).expect("Could not set up watcher");

    // MQTT setup
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_HOST)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();

    let mqtt_client = sync::Arc::new(mqtt::Client::new(create_opts).unwrap());

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(time::Duration::from_secs(MQTT_KEEP_ALIVE))
        .clean_session(true)
        .finalize();

    mqtt_client.connect(conn_opts).unwrap();

    // publisher thread!
    let pub_mqtt_client = sync::Arc::clone(&mqtt_client);
    let pub_thread = thread::spawn(move || {
        handle_messages(&rx, &pub_mqtt_client).expect("Error during handling of message");
    });

    // subscriber thread
    let sub_mqtt_client = sync::Arc::clone(&mqtt_client);
    let sub_thread = thread::spawn(move || {
        let sub = sub_mqtt_client.start_consuming();

        subscribe_topics(&sub_mqtt_client).expect("Could not subscribe to one of the topics");

        println!("Processing requests");
        for msg in sub.iter() {
            if let Some(msg) = msg {
                println!("{}", msg);
                if msg.topic() == "bar" {
                    let mut fh =
                        fs::File::create(&SUB_FILENAME).expect("Could not open file handle");
                    write!(fh, "1").expect("Could not write to sub file name");
                }
            }
        }

        println!("Quit subscriber loop");
        if sub_mqtt_client.is_connected() {
            println!("Disconnecting");
            sub_mqtt_client.unsubscribe_many(MQTT_TOPICS).unwrap();
            sub_mqtt_client.disconnect(None).unwrap();
        }
    });

    pub_thread.join().unwrap();
    sub_thread.join().unwrap();
}

enum DeviceType {
    DigitalInput,
    DigitalOutput,
    RelayOutput,
}

struct Device {
    device_type: DeviceType,
    io_group: i32,
    number: i32,
}

impl Device {
    /// file path from prefix built with device name parts
    fn path(&self, prefix: &str) -> String {
        format!(
            "{prefix}/io_group_{io_group}/{device_fmt}_{io_group}_{number:02}/{device_fmt}_value",
            prefix = prefix,
            io_group = self.io_group,
            number = self.number,
            device_fmt = match self.device_type {
                DeviceType::DigitalInput => "di",
                DeviceType::DigitalOutput => "do",
                DeviceType::RelayOutput => "ro",
            },
        )
    }
    /*

    /// Construct MQTT state topic from device name parts
    fn state_topic(&self, device_name: &str) -> &str {}

    /// Construct MQTT command topic from device name parts
    fn command_topic(&self, device_name: &str) -> &str {}
    */
}
