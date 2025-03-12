use std::fmt::Debug;

use parserc::{Input, Parser, ParserExt, next, take_till, take_while};

use crate::reader::ReadKind;

use super::ReadError;

/// Check if the `c` is whitespace.
#[inline(always)]
pub(super) fn is_ws(c: u8) -> bool {
    matches!(c, b'\x20' | b'\x09' | b'\x0d' | b'\x0a')
}

#[inline(always)]
pub(super) fn is_markup_char(c: u8) -> bool {
    matches!(c, b'<' | b'>' | b'/' | b'?' | b'\'' | b'"')
}

/// Parse `S` chars.
#[inline(always)]
pub fn parse_ws<I>(input: I) -> parserc::Result<I, I, ReadError<I>>
where
    I: Input<Item = u8> + Debug + Clone,
{
    take_while(|c: u8| is_ws(c)).parse(input)
}

/// Parse [`Eq`](https://www.w3.org/TR/xml11/#NT-Eq)
#[inline(always)]
pub fn parse_eq<I>(input: I) -> parserc::Result<(), I, ReadError<I>>
where
    I: Input<Item = u8> + Debug + Clone,
{
    let (_, input) = parse_ws(input)?;
    let span = input.clone();

    let (_, input) = next(b'=')
        .map_err(|_: ReadError<I>| ReadError::Expect(ReadKind::Eq, span.clone()))
        .parse(input)?;

    let (_, input) = parse_ws(input)?;

    Ok(((), input))
}

/// Parse quote string. see [`AttValue`](https://www.w3.org/TR/xml11/#NT-AttValue)
#[inline(always)]
pub fn parse_quote<I>(input: I) -> parserc::Result<I, I, ReadError<I>>
where
    I: Input<Item = u8> + Debug + Clone,
{
    let (double_quote, input) = next(b'"')
        .map(|_| true)
        .or(next(b'\'').map(|_| false))
        .parse(input)?;

    let end = if double_quote { b'"' } else { b'\'' };

    let (content, mut input) = take_till(|c: u8| c == end).parse(input)?;

    input.split_to(1);

    Ok((content, input))
}

#[cfg(test)]
mod tests {
    use parserc::ControlFlow;

    use crate::reader::{ReadError, ReadKind, parse_quote};

    use super::parse_eq;

    #[test]
    fn test_parse_eq() {
        assert_eq!(parse_eq(b" =<".as_slice()), Ok(((), b"<".as_slice())));

        assert_eq!(parse_eq(b" = <".as_slice()), Ok(((), b"<".as_slice())));

        assert_eq!(
            parse_eq(b" <".as_slice()),
            Err(ControlFlow::Recovable(ReadError::Expect(
                ReadKind::Eq,
                b"<".as_slice()
            )))
        );
    }

    #[test]
    fn test_quote() {
        assert_eq!(
            parse_quote(br#"'hello world'"#.as_slice()),
            Ok((b"hello world".as_slice(), b"".as_slice()))
        );

        assert_eq!(
            parse_quote(br#""hello world""#.as_slice()),
            Ok((b"hello world".as_slice(), b"".as_slice()))
        );
    }
}
