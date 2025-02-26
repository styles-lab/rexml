//! Xml document reader implementation based-on [`Event`](super::events::Event).

use parserc::{
    ControlFlow, ParseContext, ParseError, Parser, ParserExt, Span, ensure_char, ensure_keyword,
    take_while,
};

use crate::events::{Event, XmlVersion};

/// Error type returns by [`read_xml`]
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ReadError {
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),
    #[error("read `VersionInfo` error, expect {0} {1}")]
    Version(ReadKind, Span),
    #[error("read `standalone` error, expect {0} {1}")]
    Standalone(ReadKind, Span),
    #[error("read `encoding` error, expect {0} {1}")]
    Encoding(ReadKind, Span),
    #[error("read `ws` error {0}")]
    Ws(Span),
    #[error("read `eq` error {0}")]
    Eq(Span),
}

impl ParseError for ReadError {}

/// Read kind type.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ReadKind {
    #[error("`'` or `\"`")]
    Quote,
    #[error("`1.1` or `1.0`")]
    VerStr,
    #[error("`version=`")]
    LitVer,
    #[error("`yes` or `no`")]
    SDBool,
    #[error("`encoding name`")]
    EncName,
}

fn skip_ws(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let span = take_while(|c| c.is_whitespace())
        .parse(ctx)?
        .ok_or(ControlFlow::Recoverable(ReadError::Ws(ctx.span())))?;

    Ok(span)
}

fn parse_eq(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    skip_ws.ok().parse(ctx)?;
    let span = ensure_char('=')
        .fatal(ReadError::Eq(ctx.span()))
        .parse(ctx)?;
    skip_ws.ok().parse(ctx)?;
    Ok(span)
}

fn parse_version_info(ctx: &mut ParseContext<'_>) -> parserc::Result<XmlVersion, ReadError> {
    skip_ws(ctx)?;

    ensure_keyword("version")
        .fatal(ReadError::Version(ReadKind::LitVer, ctx.span()))
        .parse(ctx)?;

    parse_eq(ctx)?;

    let quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Version(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    let version = ensure_keyword("1.1")
        .map(|_| XmlVersion::Ver11)
        .or(ensure_keyword("1.0").map(|_| XmlVersion::Ver10))
        .fatal(ReadError::Version(ReadKind::VerStr, ctx.span()))
        .parse(ctx)?;

    ensure_char(if quote { '"' } else { '\'' })
        .fatal(ReadError::Version(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    Ok(version)
}

fn parse_encoding_decl<'a>(ctx: &mut ParseContext<'a>) -> parserc::Result<Span, ReadError> {
    skip_ws(ctx)?;

    ensure_keyword("encoding").parse(ctx)?;

    parse_eq(ctx)?;

    let quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Encoding(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    let quote = if quote { '"' } else { '\'' };

    let span = take_while(|c| c != quote)
        .parse(ctx)?
        .ok_or(ControlFlow::Fatal(ReadError::Encoding(
            ReadKind::EncName,
            ctx.span(),
        )))?;

    ensure_char(quote)
        .fatal(ReadError::Encoding(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    Ok(span)
}

fn parse_standalone_decl(ctx: &mut ParseContext<'_>) -> parserc::Result<bool, ReadError> {
    skip_ws(ctx)?;

    ensure_keyword("standalone").parse(ctx)?;

    parse_eq(ctx)?;

    let quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Standalone(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    let flag = ensure_keyword("yes")
        .map(|_| true)
        .or(ensure_keyword("no").map(|_| false))
        .fatal(ReadError::Standalone(ReadKind::SDBool, ctx.span()))
        .parse(ctx)?;

    ensure_char(if quote { '"' } else { '\'' })
        .fatal(ReadError::Standalone(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    Ok(flag)
}

fn parse_xml_decl<'a>(ctx: &mut ParseContext<'a>) -> parserc::Result<Event<'a>, ReadError> {
    let start = ensure_keyword("<?xml").parse(ctx)?;
    let version = parse_version_info(ctx)?;

    let encoding = parse_encoding_decl.ok().parse(ctx)?;

    let standalone = parse_standalone_decl.ok().parse(ctx)?;

    skip_ws.ok().parse(ctx)?;

    let end = ensure_keyword("?>").parse(ctx)?;

    Ok(Event::XmlDecl {
        version,
        encoding: encoding.map(|v| ctx.as_str(v).into()),
        standalone,
        span: Some(start.extend_to_inclusive(end)),
    })
}

/// Read and parse a xml document.
///
/// Processing BOM, is not the responsibility of this `fn`.
///
/// On error, returns a [`ReadError`].
pub fn read_xml<'a, D>(doc: D) -> Result<Vec<Event<'a>>, ReadError>
where
    ParseContext<'a>: From<D>,
{
    let mut ctx: ParseContext<'a> = doc.into();

    parse_xml_decl(&mut ctx).map_err(|err| err.into_raw())?;

    todo!()
}

#[cfg(test)]
mod tests {
    use parserc::{ControlFlow, ParseContext, Span};

    use crate::{
        events::{Event, XmlVersion},
        reader::{ReadError, ReadKind, parse_encoding_decl, parse_standalone_decl, parse_xml_decl},
    };

    use super::parse_version_info;

    #[test]
    fn test_version_info() {
        assert_eq!(
            parse_version_info(&mut ParseContext::from(" version='1.0'")),
            Ok(XmlVersion::Ver10)
        );

        assert_eq!(
            parse_version_info(&mut ParseContext::from(
                r#"
                    version="1.1"
                "#
            )),
            Ok(XmlVersion::Ver11)
        );

        assert_eq!(
            parse_version_info(&mut ParseContext::from(" version=")),
            Err(ControlFlow::Fatal(ReadError::Version(
                ReadKind::Quote,
                Span::new(9, 0, 1, 10)
            )))
        );
    }

    #[test]
    fn test_encoding_info() {
        assert_eq!(
            parse_encoding_decl(&mut ParseContext::from("\nencoding='utf-8'")),
            Ok(Span::new(11, 5, 2, 11))
        );

        assert_eq!(
            parse_encoding_decl(&mut ParseContext::from(r#" encoding="utf-8""#)),
            Ok(Span::new(11, 5, 1, 12))
        );
    }

    #[test]
    fn test_standalone() {
        assert_eq!(
            parse_standalone_decl(&mut ParseContext::from("\nstandalone='yes'")),
            Ok(true)
        );

        assert_eq!(
            parse_standalone_decl(&mut ParseContext::from(
                r#"
            standalone="no""#
            )),
            Ok(false)
        );
    }

    #[test]
    fn test_xml_decl() {
        assert_eq!(
            parse_xml_decl(&mut ParseContext::from(
                "<?xml version='1.1' encoding='utf-8' standalone='yes' ?>"
            )),
            Ok(Event::xml_decl_with_span(
                XmlVersion::Ver11,
                "utf-8",
                true,
                Span::new(0, 56, 1, 1)
            ))
        );

        assert_eq!(
            parse_xml_decl(&mut ParseContext::from(
                "<?xml version='1.0' standalone='yes'?>"
            )),
            Ok(Event::XmlDecl {
                version: XmlVersion::Ver10,
                encoding: None,
                standalone: Some(true),
                span: Some(Span::new(0, 38, 1, 1))
            })
        );

        assert_eq!(
            parse_xml_decl(&mut ParseContext::from(
                "<?xml version='1.0' encoding='utf-16' ?>"
            )),
            Ok(Event::XmlDecl {
                version: XmlVersion::Ver10,
                encoding: Some("utf-16".into()),
                standalone: None,
                span: Some(Span::new(0, 40, 1, 1))
            })
        );

        assert_eq!(
            parse_xml_decl(&mut ParseContext::from("<?xml version='1.0'?>")),
            Ok(Event::XmlDecl {
                version: XmlVersion::Ver10,
                encoding: None,
                standalone: None,
                span: Some(Span::new(0, 21, 1, 1))
            })
        );
    }
}
