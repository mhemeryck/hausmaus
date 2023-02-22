use hausmaus::watch;

fn main() {
    let path = "/home/mhemeryck/Projects/unipinotifiy/fixtures";
    watch(path).unwrap();
}
