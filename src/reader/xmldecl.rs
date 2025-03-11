use std::fmt::Debug;

use parserc::{AsBytes, ControlFlow, Input, Kind, Parse, Parser, ParserExt, keyword};

use crate::{
    reader::{ReadKind, parse_eq, parse_quote, parse_ws},
    types::XmlVersion,
};

use super::ReadError;

/// See [`xmldecl`](https://www.w3.org/TR/xml11/#NT-XMLDecl)
#[derive(Debug, PartialEq, Clone)]
pub struct XmlDecl<I> {
    /// required version variant.
    pub version: XmlVersion,
    /// optional encoding string.
    pub encoding: Option<I>,
    /// optional standalone flag.
    pub standalone: Option<bool>,
}

impl<I> Parse<I> for XmlDecl<I>
where
    I: Input<Item = u8> + AsBytes + Clone + Debug,
{
    type Error = ReadError<I>;

    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = keyword(b"<?xml".as_slice()).parse(input)?;

        let (s, input) = parse_ws(input)?;

        if s.len() == 0 {
            return Err(ControlFlow::Fatal(ReadError::Expect(ReadKind::S, input)));
        }

        let (_, input) = keyword(b"version".as_slice())
            .fatal()
            .map_err(|_: Kind| ReadError::Expect(ReadKind::Keyword("version"), input.clone()))
            .parse(input.clone())?;

        let (_, input) = parse_eq.fatal().parse(input)?;

        let (content, input) = parse_quote(input)?;

        let version = match content.as_bytes() {
            b"1.1" => XmlVersion::Ver11,
            b"1.0" => XmlVersion::Ver10,
            _ => {
                return Err(ControlFlow::Fatal(ReadError::Unexpect(
                    ReadKind::Version,
                    content,
                )));
            }
        };

        Ok((
            Self {
                version,
                encoding: None,
                standalone: None,
            },
            input,
        ))
    }
}
