use std::fmt::Debug;

use parserc::{AsBytes, ControlFlow, Input, Parse, Parser, ParserExt, keyword, next, take_till};

use crate::reader::{Name, parse_quote, parse_ws};

use super::{ReadError, ReadKind};

/// The start tag of an element.
///
/// See [`element`](https://www.w3.org/TR/xml11/#NT-element)
#[derive(Debug, PartialEq, Clone)]
pub struct ElemStart<I> {
    pub name: I,
    pub unparsed: I,
    pub is_empty: bool,
}

impl<I> Parse<I> for ElemStart<I>
where
    I: Input<Item = u8> + AsBytes + Debug + Clone,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = next(b'<').parse(input)?;

        let (name, mut input) = Name::into_parser().fatal().parse(input)?;

        let mut content = input.clone();

        let mut len = 0;

        loop {
            let seg;
            (seg, input) =
                take_till(|c: u8| matches!(c, b'>' | b'/' | b'"' | b'\'')).parse(input)?;

            len += seg.len();

            match input.iter().next() {
                Some(b'"') | Some(b'\'') => {
                    let quote;
                    (quote, input) = parse_quote(input)?;
                    len += quote.len() + 2;
                }
                Some(b'>') => {
                    input.split_to(1);
                    content.split_off(len);

                    return Ok((
                        Self {
                            name: name.0,
                            unparsed: content,
                            is_empty: false,
                        },
                        input,
                    ));
                }
                Some(b'/') => {
                    let (_, input) = keyword("/>").fatal().parse(input)?;
                    content.split_off(len);

                    return Ok((
                        Self {
                            name: name.0,
                            unparsed: content,
                            is_empty: true,
                        },
                        input,
                    ));
                }
                _ => {
                    return Err(ControlFlow::Fatal(ReadError::Expect(
                        ReadKind::Keyword(">"),
                        input,
                    )));
                }
            }
        }
    }
}

/// The start tag of an element.
///
/// See [`element`](https://www.w3.org/TR/xml11/#NT-element)
#[derive(Debug, PartialEq, Clone)]
pub struct ElemEnd<I> {
    pub name: I,
}

impl<I> Parse<I> for ElemEnd<I>
where
    I: Input<Item = u8> + AsBytes + Debug + Clone,
{
    type Error = ReadError<I>;

    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = keyword("</").parse(input)?;

        let (name, input) = Name::into_parser().fatal().parse(input)?;

        let (_, input) = parse_ws(input)?;

        let (_, input) = next(b'>').fatal().parse(input)?;

        Ok((Self { name: name.0 }, input))
    }
}
#[cfg(test)]
mod tests {
    use parserc::Parse;

    use crate::reader::{ElemEnd, ElemStart};

    #[test]
    fn test_el_start() {
        assert_eq!(
            ElemStart::parse(br#"<br hello="world" world="hello" >"#.as_slice()),
            Ok((
                ElemStart {
                    name: b"br".as_slice(),
                    unparsed: br#" hello="world" world="hello" "#.as_slice(),
                    is_empty: false,
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            ElemStart::parse(
                br#"<IMG align="left" src="http://www.w3.org/Icons/WWW/w3c_home" />"#.as_slice()
            ),
            Ok((
                ElemStart {
                    name: b"IMG".as_slice(),
                    unparsed: br#" align="left" src="http://www.w3.org/Icons/WWW/w3c_home" "#
                        .as_slice(),
                    is_empty: true,
                },
                b"".as_slice()
            ))
        );
    }

    #[test]
    fn test_el_end() {
        assert_eq!(
            ElemEnd::parse(br#"</br>"#.as_slice()),
            Ok((
                ElemEnd {
                    name: b"br".as_slice(),
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            ElemEnd::parse(br#"</br >"#.as_slice()),
            Ok((
                ElemEnd {
                    name: b"br".as_slice(),
                },
                b"".as_slice()
            ))
        );
    }
}
