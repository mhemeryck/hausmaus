use clap::Parser;

use hausmaus;
use hostname;
use slug;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    // Optional sysfs root path to start scanning for files
    #[arg(long, value_name = "/run/unipi")]
    sysfs: Option<String>,

    // Optional host name to pass in, used for root MQTT topic
    #[arg(long)]
    device_name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let sysfs_path: &str = cli.sysfs.as_deref().unwrap();

    //let device_name: &str = match cli.device_name.as_deref() {
    //    Some(device_name) => device_name,
    //    None => {
    //        let result = hostname::get();
    //        match result {
    //            Ok(ref os_string) => {
    //                match os_string.to_str() {
    //                    Some(str_ref) => str_ref,
    //                    None => "unknown",
    //                }
    //            },
    //            Err(_) => "unknown",
    //        }
    //    },
    //};
    //let device_name = slug::slugify(device_name);
    //println!("Device name {}", device_name);

    hausmaus::maus::run(&sysfs_path);
}
