use quick_xml::{Reader, events::Event};
use rexml::reader::read_xml;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 1000)]
fn rexml_read() {
    read_xml(include_str!("../spec/cat.svg")).unwrap();
}

#[divan::bench(sample_count = 1000)]
fn xml_dom_read() {
    xml_dom::parser::read_xml(include_str!("../spec/cat.svg")).unwrap();
}

#[divan::bench(sample_count = 1000)]
fn quic_xml_read() {
    let mut reader = Reader::from_str(include_str!("../spec/cat.svg"));

    // let mut events = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(start)) => {
                start.name().local_name();
                for attr in start.attributes() {
                    attr.unwrap();
                }
            }
            Ok(Event::Empty(empty)) => {
                empty.name().local_name();
                for attr in empty.attributes() {
                    attr.unwrap();
                }
            }
            Ok(Event::End(end)) => {
                end.name().local_name();
            }
            // There are several other `Event`s we do not consider here
            Ok(_) => {
                // events.push(event.into_owned());
            }
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
}
