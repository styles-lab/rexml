use std::fmt::Debug;

use parserc::{Input, Parse, Parser, take_till};

use crate::reader::utils::{is_markup_char, is_ws};

use super::ReadError;

/// Corresponds to dom name.
#[derive(Debug, PartialEq, Clone)]
pub struct Name<I>(pub I);

/// This parse does not check the [`NameStartChar`](https://www.w3.org/TR/xml11/#NT-NameStartChar) contraint.
impl<I> Parse<I> for Name<I>
where
    I: Input<Item = u8> + Clone + Debug,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (name, input) =
            take_till(|c: u8| is_markup_char(c) || is_ws(c) || c == b'=').parse(input)?;

        Ok((Name(name), input))
    }
}

#[cfg(test)]
mod tests {
    use parserc::Parse;

    use crate::reader::Name;

    #[test]
    fn test_name() {
        assert_eq!(
            Name::parse(b"hello:12=".as_slice()),
            Ok((Name(b"hello:12".as_slice()), b"=".as_slice()))
        );

        assert_eq!(
            Name::parse(b":12=".as_slice()),
            Ok((Name(b":12".as_slice()), b"=".as_slice()))
        );

        assert_eq!(
            Name::parse(b"12dfdd=".as_slice()),
            Ok((Name(b"12dfdd".as_slice()), b"=".as_slice()))
        );
    }
}
