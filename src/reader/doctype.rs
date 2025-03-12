use std::fmt::Debug;

use parserc::{AsBytes, ControlFlow, Input, Parse, Parser, keyword, take_till};

use crate::reader::parse_quote;

use super::{ReadError, ReadKind};

/// See [`doctype`](https://www.w3.org/TR/xml11/#NT-doctypedecl)
#[derive(Debug, PartialEq, Clone)]
pub struct DocType<I>(pub I);

impl<I> Parse<I> for DocType<I>
where
    I: Input<Item = u8> + AsBytes + Debug + Clone,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, mut input) = keyword("<!DOCTYPE").parse(input)?;

        let mut content = input.clone();

        let mut len = 0;

        let mut count = 1usize;

        loop {
            let seg;
            (seg, input) =
                take_till(|c: u8| matches!(c, b'>' | b'<' | b'"' | b'\'')).parse(input)?;

            len += seg.len();

            match input.iter().next() {
                Some(b'"') | Some(b'\'') => {
                    let quote;
                    (quote, input) = parse_quote(input)?;
                    len += quote.len() + 2;
                }
                Some(b'<') => {
                    input.split_to(1);
                    len += 1;
                    count += 1;
                }
                Some(b'>') => {
                    input.split_to(1);
                    count -= 1;

                    if count == 0 {
                        return Ok((Self(content.split_to(len)), input));
                    } else {
                        len += 1;
                    }
                }
                _ => {
                    break;
                }
            }
        }

        return Err(ControlFlow::Fatal(ReadError::Expect(
            ReadKind::Keyword(">"),
            input,
        )));
    }
}

#[cfg(test)]
mod tests {
    use parserc::Parse;

    use super::DocType;

    #[test]
    fn test_doc_type() {
        assert_eq!(
            DocType::parse(br#"<!DOCTYPE greeting SYSTEM "hello.dtd">"#.as_slice()),
            Ok((
                DocType(br#" greeting SYSTEM "hello.dtd""#.as_slice()),
                br#""#.as_slice()
            ))
        );

        assert_eq!(
            DocType::parse(
                br#"<!DOCTYPE greeting [
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    ]>"#
                .as_slice()
            ),
            Ok((
                DocType(
                    br#" greeting [
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    <!ELEMENT greeting (#PCDATA)>
                    ]"#
                    .as_slice()
                ),
                br#""#.as_slice()
            ))
        );
    }
}
