use std::fmt::Debug;

use parserc::{AsBytes, Input, Kind, Parse, Parser, ParserExt, keyword, take_till, take_until};

use super::{ReadError, ReadKind};

/// See [`chardata`](https://www.w3.org/TR/xml11/#NT-CharData)
#[derive(Debug, PartialEq, Clone)]
pub struct CharData<I>(pub I);

impl<I> Parse<I> for CharData<I>
where
    I: Input<Item = u8> + AsBytes + Debug,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (content, input) = take_till(|c| c == b'<').parse(input)?;

        Ok((CharData(content), input))
    }
}

/// See [`cdata`](https://www.w3.org/TR/xml11/#NT-CData)
#[derive(Debug, PartialEq, Clone)]
pub struct CData<I>(pub I);

impl<I> Parse<I> for CData<I>
where
    I: Input<Item = u8> + AsBytes + Debug + Clone,
{
    type Error = ReadError<I>;

    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = keyword("<![CDATA[").parse(input)?;

        let (content, mut input) = take_until("]]>")
            .fatal()
            .map_err(|_: Kind| ReadError::Expect(ReadKind::Keyword("]]>"), input.clone()))
            .parse(input.clone())?;

        input.split_to(3);

        Ok((CData(content), input))
    }
}

#[cfg(test)]
mod tests {
    use parserc::Parse;

    use crate::reader::{CData, CharData};

    #[test]
    fn test_chardata() {
        assert_eq!(
            CharData::parse(
                br#"
            hello <"#
                    .as_slice()
            ),
            Ok((
                CharData(
                    br#"
            hello "#
                        .as_slice()
                ),
                br#"<"#.as_slice()
            ))
        )
    }

    #[test]
    fn test_cdata() {
        assert_eq!(
            CData::parse(br#"<![CDATA[ >?? <? ]]>"#.as_slice()),
            Ok((CData(br#" >?? <? "#.as_slice()), b"".as_slice()))
        );
    }
}
