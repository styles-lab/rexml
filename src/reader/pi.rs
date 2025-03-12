use std::fmt::Debug;

use parserc::{AsBytes, ControlFlow, Input, Kind, Parse, Parser, ParserExt, keyword, take_until};

use crate::{
    reader::{Attr, Name, ReadKind, parse_ws},
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
    #[inline(always)]
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = keyword(b"<?xml".as_slice()).parse(input)?;

        let (version, input) = Attr::into_parser()
            .map_err(|_| ReadError::Expect(ReadKind::Version, input.clone()))
            .fatal()
            .parse(input.clone())?;

        if version.name.as_bytes() != b"version" {
            return Err(ControlFlow::Fatal(ReadError::Expect(
                ReadKind::Version,
                input,
            )));
        }

        let version = match version.value.as_bytes() {
            b"1.1" => XmlVersion::Ver11,
            b"1.0" => XmlVersion::Ver10,
            _ => {
                return Err(ControlFlow::Fatal(ReadError::Unexpect(
                    ReadKind::Version,
                    input,
                )));
            }
        };

        let (attr, input) = Attr::into_parser().ok().parse(input)?;

        let encoding = if let Some(attr) = attr {
            match attr.name.as_bytes() {
                b"encoding" => Some(attr.value),
                b"standalone" => {
                    let standalone = match attr.value.as_bytes() {
                        b"yes" => true,
                        b"no" => false,
                        _ => {
                            return Err(ControlFlow::Fatal(ReadError::Expect(
                                ReadKind::YesNo,
                                input,
                            )));
                        }
                    };

                    let (_, input) = parse_ws(input)?;

                    let (_, input) = keyword(b"?>".as_slice())
                        .fatal()
                        .map_err(|_: Kind| {
                            ReadError::Expect(ReadKind::Keyword("?>"), input.clone())
                        })
                        .parse(input.clone())?;

                    return Ok((
                        XmlDecl {
                            version,
                            encoding: None,
                            standalone: Some(standalone),
                        },
                        input,
                    ));
                }
                _ => {
                    return Err(ControlFlow::Fatal(ReadError::Expect(
                        ReadKind::Encoding,
                        input,
                    )));
                }
            }
        } else {
            None
        };

        let (attr, input) = Attr::into_parser().ok().parse(input)?;

        let standalone = if let Some(attr) = attr {
            if attr.name.as_bytes() == b"standalone" {
                let standalone = match attr.value.as_bytes() {
                    b"yes" => true,
                    b"no" => false,
                    _ => {
                        return Err(ControlFlow::Fatal(ReadError::Expect(
                            ReadKind::YesNo,
                            input,
                        )));
                    }
                };

                Some(standalone)
            } else {
                return Err(ControlFlow::Fatal(ReadError::Expect(
                    ReadKind::Keyword("standalone"),
                    input,
                )));
            }
        } else {
            None
        };

        let (_, input) = parse_ws(input)?;

        let (_, input) = keyword(b"?>".as_slice())
            .fatal()
            .map_err(|_: Kind| ReadError::Expect(ReadKind::Keyword("?>"), input.clone()))
            .parse(input.clone())?;

        return Ok((
            XmlDecl {
                version,
                encoding,
                standalone,
            },
            input,
        ));
    }
}

/// See [`pi`](https://www.w3.org/TR/xml11/#NT-PI)
#[derive(Debug, PartialEq, Clone)]
pub struct PI<I> {
    pub name: I,
    /// unparsed content.
    pub unparsed: I,
}

impl<I> Parse<I> for PI<I>
where
    I: Input<Item = u8> + AsBytes + Clone + Debug,
{
    type Error = ReadError<I>;
    fn parse(input: I) -> parserc::Result<Self, I, Self::Error> {
        let (_, input) = keyword("<?").parse(input)?;

        let (name, input) = Name::into_parser().fatal().parse(input)?;

        let (unparsed, mut input) = take_until("?>")
            .fatal()
            .map_err(|_: Kind| ReadError::Expect(ReadKind::Keyword("?>"), input.clone()))
            .parse(input.clone())?;

        input.split_to(2);

        Ok((
            Self {
                name: name.0,
                unparsed,
            },
            input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use parserc::Parse;

    use crate::{
        reader::{PI, XmlDecl},
        types::XmlVersion,
    };

    #[test]
    fn test_xmldecl() {
        assert_eq!(
            XmlDecl::parse(br#"<?xml version="1.1"?>"#.as_slice()),
            Ok((
                XmlDecl {
                    version: XmlVersion::Ver11,
                    encoding: None,
                    standalone: None
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            XmlDecl::parse(br#"<?xml version="1.1" standalone='yes'?>"#.as_slice()),
            Ok((
                XmlDecl {
                    version: XmlVersion::Ver11,
                    encoding: None,
                    standalone: Some(true)
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            XmlDecl::parse(br#"<?xml version="1.1" encoding="UTF-8" ?>"#.as_slice()),
            Ok((
                XmlDecl {
                    version: XmlVersion::Ver11,
                    encoding: Some(b"UTF-8".as_slice()),
                    standalone: None
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            XmlDecl::parse(br#"<?xml version="1.1" encoding="UTF-8" standalone='no'?>"#.as_slice()),
            Ok((
                XmlDecl {
                    version: XmlVersion::Ver11,
                    encoding: Some(b"UTF-8".as_slice()),
                    standalone: Some(false)
                },
                b"".as_slice()
            ))
        );
    }

    #[test]
    fn test_pi() {
        assert_eq!(
            PI::parse(br#"<?xml version="1.1"?>"#.as_slice()),
            Ok((
                PI {
                    name: b"xml".as_slice(),
                    unparsed: br#" version="1.1""#.as_slice()
                },
                b"".as_slice()
            ))
        );

        assert_eq!(
            PI::parse(br#"<?xml version='1.1'  ?>"#.as_slice()),
            Ok((
                PI {
                    name: b"xml".as_slice(),
                    unparsed: br#" version='1.1'  "#.as_slice()
                },
                b"".as_slice()
            ))
        );
    }
}
