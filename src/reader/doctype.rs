use parserc::{ParseContext, Parser, ParserExt, Span, ensure_keyword};

use crate::reader::{
    ReadKind,
    misc::{quote, skip_ws},
};

use super::ReadError;

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

    skip_ws
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

        skip_ws
            .fatal(ReadError::ExternalId(ReadKind::Ws, start))
            .parse(ctx)?;

        let system_id_literal = quote(|_| Ok(()))
            .map_err(|_| ReadError::ExternalId(ReadKind::SystemLiteral, start))
            .parse(ctx)?;

        Ok(ExternalId::Public(pub_id_literal, system_id_literal))
    }
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::doctype::{ExternalId, parse_external_id};

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
}
