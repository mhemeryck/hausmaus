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

    #[arg(long)]
    debug: bool,
}

// device name from hostname
fn device_name() -> Option<String> {
    match hostname::get() {
        Ok(os_string) => match os_string.into_string() {
            Ok(str_ref) => Some(str_ref),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn main() {
    let cli = Cli::parse();

    let sysfs_path: &str = cli.sysfs.as_deref().unwrap();

    let device_name: String = match cli.device_name.as_deref() {
        // from input arg
        Some(device_name) => device_name.to_string(),
        // from hostname
        None => device_name().unwrap(),
    };
    let device_name = slug::slugify(device_name);
    let device_name = device_name.as_str();

    let debug = cli.debug;

    hausmaus::maus::run(&sysfs_path, &device_name, debug);
}
