use parserc::Parse;
use rexml::reader::{Name, parse_eq, parse_ws};

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_name() {
    Name::parse(b"hello:12=".as_slice()).unwrap();
}

#[divan::bench]
fn bench_eq() {
    parse_eq(b" =<".as_slice()).unwrap();
}

#[divan::bench]
fn bench_ws() {
    parse_ws(b"    ".as_slice()).unwrap();
}
