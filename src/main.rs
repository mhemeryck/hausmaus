use hausmaus::watch;

const PATH: &str = "/home/mhemeryck/Projects/unipinotifiy/fixtures";

fn main() {
    watch(PATH).unwrap();
}
