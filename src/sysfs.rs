/// sysfs contains the interface the file system based view on IO
//pub mod read;
//pub mod write;

pub type FileEvent = (std::sync::Arc<Device>, bool, std::time::Duration);

use crate::device::Device;
use std;

/// Crawls a directory structure for filenames matching given input
pub fn crawl(
    dir: &std::path::Path,
    module_name: &str,
    devices: &mut std::vec::Vec<crate::device::Device>,
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
                crawl(&path, module_name, devices)?;
            } else {
                if let Some(path_str) = path.to_str() {
                    // The id we use here is just the current length of the list
                    let id: u8 = devices.len().try_into().unwrap();
                    if let Ok(device) = Device::from_path(id, path_str, module_name) {
                        devices.push(device);
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path;
    use std::vec;
    use tempdir;

    const FILENAME_PATTERN: &str =
        r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";

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
        let module_name = "foo";
        let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/di_value";
        if let Some(device) = device_from_path(&module_name, &re, &path) {
            assert_eq!(device.module_name, "foo");
            assert_eq!(device.number, 7);
            assert_eq!(device.io_group, 2);
            assert_eq!(device.device_type, DeviceType::DigitalInput);
        } else {
            panic!("Could not find a device from path");
        }
    }

    #[test]
    fn test_device_from_path_not_found() {
        let module_name = "foo";
        let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/foo";
        if let Some(_) = device_from_path(&module_name, &re, &path) {
            panic!("It shouldn't find a device in this case!");
        }
    }
}
