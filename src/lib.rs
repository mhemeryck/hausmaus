use notify::{self, Watcher};
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::time;
use std::vec;

const FILENAME: &str = "ro_value";

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
    let path = path::Path::new(&path_str);
    let mut paths: vec::Vec<path::PathBuf> = vec::Vec::new();
    crawl(&path, &mut paths, FILENAME).unwrap();

    let (tx, rx) = sync::mpsc::channel();

    let config = notify::Config::default()
        .with_poll_interval(time::Duration::from_millis(250))
        .with_compare_contents(true);

    let mut watcher = notify::PollWatcher::new(tx, config)?;
    // watcher.watch(path_str.as_ref(), notify::RecursiveMode::Recursive)?;

    for path in paths.iter() {
        println!("Path {:?}", path.canonicalize().unwrap());
        watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;
    }

    for res in rx {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                println!("event kind: {:?}", event.kind);
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
