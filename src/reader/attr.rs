use std::fmt::Debug;

use parserc::{ControlFlow, Input, Parse, Parser, ParserExt};

use crate::reader::{Name, ReadKind, parse_eq, parse_quote, parse_ws};

use super::ReadError;

/// Attribute value pair.
#[derive(Debug, PartialEq, Clone)]
pub struct Attr<I> {
    pub name: I,
    pub value: I,
}

impl<I> Parse<I> for Attr<I>
where
    I: Input<Item = u8> + Debug + Clone,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (s, input) = parse_ws(input)?;

        if s.is_empty() {
            return Err(ControlFlow::Recovable(ReadError::Expect(
                ReadKind::S,
                input,
            )));
        }

        let (name, input) = Name::into_parser().parse(input)?;

        let (_, input) = parse_eq.fatal().parse(input)?;

        let (value, input) = parse_quote.fatal().parse(input)?;

        Ok((
            Self {
                name: name.0,
                value,
            },
            input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use parserc::Parse;

    use crate::reader::Attr;

    #[test]
    fn test_attr() {
        assert_eq!(
            Attr::parse(b" value='hello world'".as_slice()),
            Ok((
                Attr {
                    name: b"value".as_slice(),
                    value: b"hello world".as_slice(),
                },
                b"".as_slice()
            ))
        );
    }
}
