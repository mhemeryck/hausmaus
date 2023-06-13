/// sysfs contains the interface the file system based view on IO
pub mod read;
pub mod write;

pub type FileEvent = (u8, bool, std::time::Duration);

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use std::fs;
//    use std::io::Write;
//    use std::path;
//    use std::vec;
//    use tempdir;
//
//    const FILENAME_PATTERN: &str =
//        r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";
//
//    #[test]
//    fn test_crawl_simple_file_no_match() {
//        let tmp_dir =
//            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
//        let path = tmp_dir.path().join("myfile.txt");
//        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
//        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");
//
//        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
//
//        let re = regex::Regex::new("foo").unwrap();
//        crawl(tmp_dir.path(), &re, &mut paths).expect("Expect crawl to work");
//
//        assert_eq!(paths.len(), 0);
//
//        tmp_dir.close().unwrap();
//    }
//
//    #[test]
//    fn test_crawl_file_match() {
//        let tmp_dir =
//            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
//        let path = tmp_dir.path().join("myfile.txt");
//        let mut tmp_file = fs::File::create(&path).expect("Could not open a new temp file");
//        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");
//
//        let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
//
//        let re = regex::Regex::new("myfile.txt").unwrap();
//        crawl(tmp_dir.path(), &re, &mut paths).expect("Expect crawl to work");
//
//        assert_eq!(paths.len(), 1);
//        assert_eq!(paths[0], path);
//
//        tmp_dir.close().unwrap();
//    }
//
//    #[test]
//    fn test_device_from_path() {
//        let module_name = "foo";
//        let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
//        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/di_value";
//        if let Some(device) = device_from_path(&module_name, &re, &path) {
//            assert_eq!(device.module_name, "foo");
//            assert_eq!(device.number, 7);
//            assert_eq!(device.io_group, 2);
//            assert_eq!(device.device_type, DeviceType::DigitalInput);
//        } else {
//            panic!("Could not find a device from path");
//        }
//    }
//
//    #[test]
//    fn test_device_from_path_not_found() {
//        let module_name = "foo";
//        let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
//        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/foo";
//        if let Some(_) = device_from_path(&module_name, &re, &path) {
//            panic!("It shouldn't find a device in this case!");
//        }
//    }
//}
