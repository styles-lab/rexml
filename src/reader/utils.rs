use std::fmt::Debug;

use parserc::{Input, Parser, ParserExt, next, take_while};

use crate::reader::ReadKind;

use super::ReadError;

/// Check if the `c` is whitespace.#[inline(always)]
#[inline(always)]
pub(super) fn is_ws(c: u8) -> bool {
    matches!(c, b'\x20' | b'\x09' | b'\x0d' | b'\x0a')
}

#[inline(always)]
pub(super) fn is_markup_char(c: u8) -> bool {
    matches!(c, b'<' | b'>' | b'/' | b'?' | b'\'' | b'"')
}

#[inline(always)]
pub fn parse_ws<I>(input: I) -> parserc::Result<I, I, ReadError<I>>
where
    I: Input<Item = u8> + Debug + Clone,
{
    take_while(|c: u8| is_ws(c)).parse(input)
}

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

#[cfg(test)]
mod tests {
    use parserc::ControlFlow;

    use crate::reader::{ReadError, ReadKind};

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
}
