use notify::{self, Watcher};
use paho_mqtt as mqtt;
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::time;
use std::vec;

const FILENAME: &str = "ro_value";
const POLL_INTERVAL: u64 = 250;

const MQTT_HOST: &str = "tcp://emqx.mhemeryck.com";
const MQTT_CLIENT_ID: &str = "hausmaus";
const MQTT_KEEP_ALIVE: u64 = 20;

const MQTT_TOPIC: &str = "foo";
// const MQTT_PAYLOAD: &[u8; 6] = b"Hello!";
const MQTT_QOS: i32 = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
fn setup_watcher(path_str: &str, watcher: &mut notify::PollWatcher) {
    // Paths
    let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
    crawl(&path::Path::new(&path_str), &mut paths, FILENAME).unwrap();

    for path in paths.iter() {
        println!("Path {:?}", path.canonicalize().unwrap());
        watcher
            .watch(path.as_ref(), notify::RecursiveMode::NonRecursive)
            .unwrap();
    }
}

/// handle_messages receives any file events and sends them out over MQTT
fn handle_messages(
    rx: &sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    mqtt_client: &mqtt::Client,
) {
    for res in rx {
        if let Ok(event) = res {
            for path in event.paths {
                let message =
                    mqtt::Message::new(MQTT_TOPIC, path.to_str().unwrap().as_bytes(), MQTT_QOS);
                mqtt_client.publish(message).unwrap();
            }
            //println!("changed: {:?}", event);
            //println!("event kind: {:?}", event.kind);
        }
    }
}
pub fn main(path_str: &str) {
    let (tx, rx): (
        sync::mpsc::Sender<Result<notify::Event, notify::Error>>,
        sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) = sync::mpsc::channel();

    // PollWatcher setup
    let config = notify::Config::default()
        .with_poll_interval(time::Duration::from_millis(POLL_INTERVAL))
        .with_compare_contents(true);
    let mut watcher = notify::PollWatcher::new(tx, config).unwrap();
    setup_watcher(&path_str, &mut watcher);

    // MQTT setup
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_HOST)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();

    let mqtt_client = mqtt::Client::new(create_opts).unwrap();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(time::Duration::from_secs(MQTT_KEEP_ALIVE))
        .clean_session(true)
        .finalize();

    mqtt_client.connect(conn_opts).unwrap();

    handle_messages(&rx, &mqtt_client);
}
