use clap;

use hausmaus;

const PATH: &str = "/run/unipi";

fn main() {
    // CLI args
    let matches = clap::Command::new("hausmaus")
        .arg(
            clap::Arg::new("sysfs")
                .default_value(PATH)
                .long("sysfs-path")
                .help("SysFS scan path"),
        )
        .get_matches();
    let sysfs_path = matches.get_one::<String>("sysfs").unwrap();

    hausmaus::maus::run(&sysfs_path);
}
