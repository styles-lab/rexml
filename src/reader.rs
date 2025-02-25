//! Xml document reader implementation based-on [`Event`](super::events::Event).

use parserc::{
    ParseContext, ParseError, Parser, ParserExt, Span, ensure_char, ensure_keyword, take_while,
};

use crate::events::{Event, XmlVersion};

/// Error type returns by [`read_xml`]
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ReadError {
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),
    #[error("read `VersionInfo` error, expect {0} {1}")]
    Version(ReadKind, Span),
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
}

fn skip_ws(ctx: &mut ParseContext<'_>) -> parserc::Result<(), ReadError> {
    take_while(|c| c.is_whitespace()).parse(ctx).map(|_| ())
}

fn parse_version_info(ctx: &mut ParseContext<'_>) -> parserc::Result<XmlVersion, ReadError> {
    skip_ws(ctx)?;
    ensure_keyword("version=")
        .fatal(ReadError::Version(ReadKind::LitVer, ctx.span()))
        .parse(ctx)?;

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

fn parse_xml_decl<'a>(ctx: &mut ParseContext<'a>) -> parserc::Result<Event<'a>, ReadError> {
    let start = ensure_keyword("<?xml").parse(ctx)?;
    let version = parse_version_info(ctx)?;

    let end = ensure_keyword("?>").parse(ctx)?;

    Ok(Event::XmlDecl {
        version,
        encoding: None,
        standalone: None,
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
        events::XmlVersion,
        reader::{ReadError, ReadKind},
    };

    use super::parse_version_info;

    #[test]
    fn test_version_info() {
        assert_eq!(
            parse_version_info(&mut ParseContext::from("version='1.0'")),
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
            parse_version_info(&mut ParseContext::from("version=")),
            Err(ControlFlow::Fatal(ReadError::Version(
                ReadKind::Quote,
                Span::new(8, 0, 1, 9)
            )))
        );
    }
}
