/// sysfs contains the interface the file system based view on IO
pub mod read;

pub type FileEvent = (std::sync::Arc<Device>, bool, std::time::Duration);

use crate::device::{Device, DeviceType};
use std;

const FILENAME_PATTERN: &str =
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value";

/// Crawls a directory structure for filenames matching given input
pub fn crawl(
    dir: &std::path::Path,
    filename_regex: &regex::Regex,
    paths: &mut std::vec::Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        log::debug!("Checking dir {:?}", dir);
        for entry in std::fs::read_dir(dir)? {
            let entry: std::fs::DirEntry = entry?;
            let path = entry.path();
            log::debug!("Checking path {:?}", path);
            // Skip symlinks to avoid infinite loops
            if path.is_symlink() {
                continue;
            }

            // dirs need to be crawled further
            if path.is_dir() {
                crawl(&path, filename_regex, paths)?;
            } else {
                match path.to_str() {
                    Some(path_str) => {
                        if filename_regex.is_match(path_str) {
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

/// Create a device from a path string
fn device_from_path(name_str: &str, path_str: &str) -> Option<Device> {
    let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
    if let Some(captures) = re.captures(path_str) {
        if let (Some(device_fmt), Some(io_group_str), Some(number_str)) = (
            captures.name("device_fmt"),
            captures.name("io_group"),
            captures.name("number"),
        ) {
            // Map against device type from capture
            let device_type = match device_fmt.as_str() {
                "di" => DeviceType::DigitalInput,
                "do" => DeviceType::DigitalOutput,
                "ro" => DeviceType::RelayOutput,
                _ => return None,
            };

            // Parse and cast from capture
            if let (Ok(io_group), Ok(number)) = (
                io_group_str.as_str().parse::<i32>(),
                number_str.as_str().parse::<i32>(),
            ) {
                let name = name_str.to_string();
                return Some(Device {
                    name,
                    device_type,
                    io_group,
                    number,
                });
            }
        }
    }
    // In all other cases, nothing was found
    None
}

/// Build a list of devices from a list paths
pub fn devices_from_paths(
    name_str: &str,
    paths: &std::vec::Vec<std::path::PathBuf>,
    devices: &mut std::vec::Vec<crate::device::Device>,
) {
    for path in paths {
        if let Some(path_str) = path.to_str() {
            if let Some(device) = device_from_path(&name_str, path_str) {
                devices.push(device);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path;
    use std::vec;
    use tempdir;

    #[test]
    fn test_crawl_simple_file_no_match() {
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        let path = tmp_dir.path().join("myfile.txt");
        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();

        let re = regex::Regex::new("foo").unwrap();
        crawl(tmp_dir.path(), &re, &mut paths).expect("Expect crawl to work");

        assert_eq!(paths.len(), 0);

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_crawl_file_match() {
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        let path = tmp_dir.path().join("myfile.txt");
        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();

        let re = regex::Regex::new("myfile.txt").unwrap();
        crawl(tmp_dir.path(), &re, &mut paths).expect("Expect crawl to work");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], path);

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_device_from_path() {
        let name = "foo";
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/di_value";
        if let Some(device) = device_from_path(&name, &path) {
            assert_eq!(device.name, "foo");
            assert_eq!(device.number, 7);
            assert_eq!(device.io_group, 2);
            assert_eq!(device.device_type, DeviceType::DigitalInput);
        } else {
            panic!("Could not find a device from path");
        }
    }

    #[test]
    fn test_device_from_path_not_found() {
        let name = "foo";
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/foo";
        if let Some(_) = device_from_path(&name, &path) {
            panic!("It shouldn't find a device in this case!");
        }
    }
}
