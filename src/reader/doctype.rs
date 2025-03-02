use parserc::{ControlFlow, FromSrc, ParseContext, Parser, ParserExt, ensure_keyword, take_till};

use super::{DocType, ReadError, ReadKind, misc::quote};

impl FromSrc for DocType {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
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
                            return Ok(Self(start.extend_to_inclusive(span)));
                        }
                    }
                    _ => {
                        quote(|_| Ok(()))
                            .fatal(ReadError::DocType(ReadKind::Quote, ctx.span()))
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
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext, Span};

    use crate::reader::DocType;

    #[test]
    fn test_doc_type() {
        assert_eq!(
            DocType::parse(&mut ParseContext::from(
                r#"<!DOCTYPE greeting SYSTEM "hello.dtd">"#
            )),
            Ok(DocType(Span::new(0, 38, 1, 1)))
        );

        assert_eq!(
            DocType::parse(&mut ParseContext::from(
                r#"<!DOCTYPE greeting [
                   <!ELEMENT greeting (#PCDATA)>
                ]>"#
            )),
            Ok(DocType(Span::new(0, 88, 1, 1)))
        );
    }
}
