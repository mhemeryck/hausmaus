use notify::{self, Watcher};
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::time;
use std::vec;

const FILENAME: &str = "ro_value";
const POLL_INTERVAL: u64 = 250;

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

/// watch watches a folder for changes and prints them (for now)
pub fn watch(path_str: &str) -> notify::Result<()> {
    // Paths
    let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
    crawl(&path::Path::new(&path_str), &mut paths, FILENAME).unwrap();

    let (tx, rx) = sync::mpsc::channel();

    let config = notify::Config::default()
        .with_poll_interval(time::Duration::from_millis(POLL_INTERVAL))
        .with_compare_contents(true);

    let mut watcher = notify::PollWatcher::new(tx, config)?;

    for path in paths.iter() {
        println!("Path {:?}", path.canonicalize().unwrap());
        watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;
    }

    for res in rx {
        if let Ok(event) = res {
            println!("changed: {:?}", event);
            println!("event kind: {:?}", event.kind);
        }
    }
    Ok(())
}
