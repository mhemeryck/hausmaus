use notify::{self, Watcher};
use regex::Regex;
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::time;
use std::vec;

const PATH: &str = "/home/mhemeryck/Projects/hausmaus/fixtures";
const FILENAME_PATTERN: &str =
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro_value)";
const POLL_INTERVAL: u64 = 200;

/// Crawls a directory structure for filenames matching given input
fn crawl(
    dir: &path::Path,
    filename_regex: &Regex,
    paths: &mut vec::Vec<path::PathBuf>,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry: fs::DirEntry = entry?;
            let path = entry.path();
            if path.is_dir() {
                crawl(&path, filename_regex, paths)?;
            } else if !path.is_symlink() {
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

fn main() {
    // Crawl a folder for paths to watch based on a regex
    let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
    let re = Regex::new(FILENAME_PATTERN).unwrap();

    let (tx, rx) = sync::mpsc::channel();

    // PollWatcher setup
    let config = notify::Config::default()
        .with_poll_interval(time::Duration::from_millis(POLL_INTERVAL))
        .with_compare_contents(true);
    let mut watcher = notify::PollWatcher::new(tx, config).unwrap();

    match crawl(&path::Path::new(&PATH), &re, &mut paths) {
        Ok(_) => {
            for path in paths.iter() {
                println!("Found path: {:?}", path);
                // Set up watcher
                watcher
                    .watch(path.as_ref(), notify::RecursiveMode::NonRecursive)
                    .unwrap();
            }
        }
        Err(error) => {
            panic!("PANIC: {:?}!", error);
        }
    }

    for res in rx {
        if let Ok(event) = res {
            println!("changed: {:?}", event);
            println!("event kind: {:?}", event.kind);
            for path in event.paths {
                if let Some(path_str) = path.to_str() {
                    println!("path: {:?}", path_str);
                }
            }
        }
    }
}
