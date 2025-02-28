use parserc::{
    ControlFlow, IntoParser, ParseContext, Parser, ParserExt, Span, ensure_keyword, take_till,
};

use crate::reader::{ReadKind, misc::quote};

use super::{ReadError, WS};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum ExternalId {
    System(Span),
    Public(Span, Span),
}

pub(super) fn parse_external_id(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<ExternalId, ReadError> {
    let start = ctx.span();

    let system = ensure_keyword("SYSTEM")
        .map(|_| true)
        .or(ensure_keyword("PUBLIC").map(|_| false))
        .map_err(|_: ReadError| ReadError::ExternalId(ReadKind::ExternalType, start))
        .parse(ctx)?;

    WS::into_parser()
        .fatal(ReadError::ExternalId(ReadKind::Ws, start))
        .parse(ctx)?;

    let start = ctx.span();
    if system {
        let system_id_literal = quote(|_| Ok(()))
            .map_err(|_| ReadError::ExternalId(ReadKind::SystemLiteral, start))
            .parse(ctx)?;

        Ok(ExternalId::System(system_id_literal))
    } else {
        let pub_id_literal = quote(|c| {
            const MATCHES: &[char] = &[
                '-', '\'', '(', ')', '+', ',', '.', '/', ':', '=', '?', ';', '!', '*', '#', '@',
                '$', '_', '%',
            ];

            if c.is_whitespace()
                || c.is_ascii_alphanumeric()
                || MATCHES.iter().find(|v| **v == c).is_some()
            {
                Ok(())
            } else {
                return Err(ReadError::ExternalId(ReadKind::PubIdLiteral, start));
            }
        })
        .parse(ctx)?;

        let start = ctx.span();

        WS::into_parser()
            .fatal(ReadError::ExternalId(ReadKind::Ws, start))
            .parse(ctx)?;

        let system_id_literal = quote(|_| Ok(()))
            .map_err(|_| ReadError::ExternalId(ReadKind::SystemLiteral, start))
            .parse(ctx)?;

        Ok(ExternalId::Public(pub_id_literal, system_id_literal))
    }
}

pub(super) fn parse_doc_type_decl(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let span = ctx.span();

    let start = ensure_keyword("<!DOCTYPE")
        .map_err(|_: ReadError| ReadError::DocType(ReadKind::Prefix("<!DOCTYPE"), span))
        .parse(ctx)?;

    let mut c = 1;

    loop {
        take_till(|c| matches!(c, '<' | '>' | '"' | '\'')).parse(ctx)?;

        let (next, span) = ctx.peek();

        if let Some(next) = next {
            match next {
                '<' => {
                    ctx.next();
                    c += 1;
                }
                '>' => {
                    ctx.next();
                    c -= 1;

                    if c == 0 {
                        return Ok(start.extend_to_inclusive(span));
                    }
                }
                _ => {
                    quote(|_| Ok(()))
                        .fatal(ReadError::DocType(ReadKind::Quote, span))
                        .parse(ctx)?;
                }
            }
        } else {
            break;
        }
    }

    return Err(ControlFlow::Fatal(ReadError::DocType(
        ReadKind::Suffix(">"),
        span,
    )));
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::doctype::{ExternalId, parse_doc_type_decl, parse_external_id};

    #[test]
    fn test_external_id() {
        assert_eq!(
            parse_external_id(&mut ParseContext::from(
                r#"SYSTEM "http://www.textuality.com/boilerplate/OpenHatch.xml""#
            )),
            Ok(ExternalId::System(Span::new(8, 51, 1, 9)))
        );

        assert_eq!(
            parse_external_id(&mut ParseContext::from(
                r#"PUBLIC '-//Textuality//TEXT Standard open-hatch boilerplate//EN'
                "http://www.textuality.com/boilerplate/OpenHatch.xml"
                "#
            )),
            Ok(ExternalId::Public(
                Span::new(8, 55, 1, 9),
                Span::new(82, 51, 2, 18)
            ))
        );
    }

    #[test]
    fn test_doc_type() {
        assert_eq!(
            parse_doc_type_decl(&mut ParseContext::from(
                r#"<!DOCTYPE greeting SYSTEM "hello.dtd">"#
            )),
            Ok(Span::new(0, 38, 1, 1))
        );

        assert_eq!(
            parse_doc_type_decl(&mut ParseContext::from(
                r#"<!DOCTYPE greeting [
                   <!ELEMENT greeting (#PCDATA)>
                ]>"#
            )),
            Ok(Span::new(0, 88, 1, 1))
        );
    }
}
