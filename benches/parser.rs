use parserc::Parse;
use rexml::reader::{Attr, Name, parse_eq, parse_quote, parse_ws};

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

#[divan::bench]
fn bench_quote() {
    parse_quote(br#""hello:12=""#.as_slice()).unwrap();
}

#[divan::bench]
fn bench_attr() {
    Attr::parse(b" value='hello world'".as_slice()).unwrap();
}
