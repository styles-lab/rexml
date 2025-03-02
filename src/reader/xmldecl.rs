use parserc::{
    ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Span, ensure_char,
    ensure_keyword, take_while,
};

use crate::types::XmlVersion;

use super::{ReadError, ReadKind, WS, XmlDecl, misc::parse_eq};

fn parse_version_info(ctx: &mut ParseContext<'_>) -> parserc::Result<XmlVersion, ReadError> {
    WS::parse(ctx)?;

    ensure_keyword("version").parse(ctx)?;

    parse_eq(ctx)?;

    let quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Version(
            ReadKind::Prefix("`\"` or `'`"),
            ctx.span(),
        ))
        .parse(ctx)?;

    let version = ensure_keyword("1.1")
        .map(|_| XmlVersion::Ver11)
        .or(ensure_keyword("1.0").map(|_| XmlVersion::Ver10))
        .fatal(ReadError::Version(ReadKind::LitVersion, ctx.span()))
        .parse(ctx)?;

    ensure_char(if quote { '"' } else { '\'' })
        .fatal(ReadError::Version(
            ReadKind::Suffix(if quote { "\"" } else { "'" }),
            ctx.span(),
        ))
        .parse(ctx)?;

    Ok(version)
}

fn parse_encoding_decl<'a>(ctx: &mut ParseContext<'a>) -> parserc::Result<Span, ReadError> {
    WS::parse(ctx)?;

    ensure_keyword("encoding").parse(ctx)?;

    parse_eq(ctx)?;

    let double_quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Encoding(
            ReadKind::Prefix("`\"` or `'`"),
            ctx.span(),
        ))
        .parse(ctx)?;

    let quote = if double_quote { '"' } else { '\'' };

    let span = take_while(|c| c != quote)
        .parse(ctx)?
        .ok_or(ControlFlow::Fatal(ReadError::Encoding(
            ReadKind::EncName,
            ctx.span(),
        )))?;

    ensure_char(quote)
        .fatal(ReadError::Encoding(
            ReadKind::Suffix(if double_quote { "\"" } else { "'" }),
            ctx.span(),
        ))
        .parse(ctx)?;

    Ok(span)
}

fn parse_standalone_decl(ctx: &mut ParseContext<'_>) -> parserc::Result<bool, ReadError> {
    WS::parse(ctx)?;

    ensure_keyword("standalone").parse(ctx)?;

    parse_eq(ctx)?;

    let double_quote = ensure_char('\'')
        .map(|_| false)
        .or(ensure_char('"').map(|_| true))
        .fatal(ReadError::Standalone(
            ReadKind::Prefix("`\"` or `'`"),
            ctx.span(),
        ))
        .parse(ctx)?;

    let flag = ensure_keyword("yes")
        .map(|_| true)
        .or(ensure_keyword("no").map(|_| false))
        .fatal(ReadError::Standalone(
            ReadKind::LitStr("`yes` or `no`"),
            ctx.span(),
        ))
        .parse(ctx)?;

    ensure_char(if double_quote { '"' } else { '\'' })
        .fatal(ReadError::Standalone(
            ReadKind::Suffix(if double_quote { "\"" } else { "'" }),
            ctx.span(),
        ))
        .parse(ctx)?;

    Ok(flag)
}

impl FromSrc for XmlDecl {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        ensure_keyword("<?xml").parse(ctx)?;

        let version = parse_version_info(ctx)?;

        let encoding = parse_encoding_decl.ok().parse(ctx)?;

        let standalone = parse_standalone_decl.ok().parse(ctx)?;

        WS::into_parser().ok().parse(ctx)?;

        ensure_keyword("?>")
            .fatal(ReadError::XmlDecl(ReadKind::Suffix("?>"), ctx.span()))
            .parse(ctx)?;

        Ok(Self {
            version,
            encoding,
            standalone,
        })
    }
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
                ReadKind::Prefix("`\"` or `'`"),
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
}
