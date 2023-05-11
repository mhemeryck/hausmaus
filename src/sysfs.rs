/// sysfs contains the interface the file system based view on IO
pub mod read;

pub type FileEvent = (bool, std::time::Duration);

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
}
