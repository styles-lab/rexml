use parserc::Parse;
use rexml::reader::{
    Attr, CData, CharData, Comment, DocType, Name, PI, XmlDecl, parse_eq, parse_quote, parse_ws,
};

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

#[divan::bench]
fn bench_xml_decl() {
    XmlDecl::parse(br#"<?xml version="1.1" encoding="UTF-8" standalone='no'?>"#.as_slice())
        .unwrap();
}

#[divan::bench]
fn bench_pi() {
    PI::parse(br#"<?xml version="1.1" encoding="UTF-8" standalone='no'?>"#.as_slice()).unwrap();
}

#[divan::bench]
fn bench_chardata() {
    CharData::parse(
        br#"
            hello <"#
            .as_slice(),
    )
    .unwrap();
}

#[divan::bench]
fn bench_cdata() {
    CData::parse(br#"<![CDATA[ >?? <? ]]>"#.as_slice()).unwrap();
}

#[divan::bench]
fn bench_comment() {
    Comment::parse(br#"<!-- >?? <? -->"#.as_slice()).unwrap();
}

#[divan::bench]
fn bench_doc_type() {
    DocType::parse(
        br#"<!DOCTYPE greeting [
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    ]>"#
        .as_slice(),
    )
    .unwrap();
}
