use parserc::{
    ControlFlow, ParseContext, Parser, ParserExt, Span, ensure_char, ensure_keyword, take_while,
};

use crate::events::{Event, XmlVersion};

use super::{
    ReadError, ReadKind,
    misc::{parse_eq, skip_ws},
};

pub(super) fn parse_version_info(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<XmlVersion, ReadError> {
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

pub(super) fn parse_encoding_decl<'a>(
    ctx: &mut ParseContext<'a>,
) -> parserc::Result<Span, ReadError> {
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

pub(super) fn parse_standalone_decl(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<bool, ReadError> {
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

pub(super) fn parse_xml_decl<'a>(
    ctx: &mut ParseContext<'a>,
) -> parserc::Result<Event<'a>, ReadError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
