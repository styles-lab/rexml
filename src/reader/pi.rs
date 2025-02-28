use parserc::{
    ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Span, ensure_keyword,
    take_till, take_while,
};

use super::{PI, ReadError, ReadKind, WS, misc::parse_name};

impl FromSrc for PI {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();
        ensure_keyword("<?")
            .map_err(|_: ReadError| ReadError::PI(ReadKind::Prefix("<?"), start))
            .parse(ctx)?;

        let span = ctx.span();

        let target = parse_name
            .map_err(|_| ReadError::PI(ReadKind::PITarget, span))
            .parse(ctx)?;

        // check reserved word `('X' | 'x') ('M' | 'm') ('L' | 'l')`
        if ctx.as_str(target).to_lowercase() == "xml" {
            return Err(ControlFlow::Fatal(ReadError::PI(
                ReadKind::ReservedXml,
                target,
            )));
        }

        if let Some(_) = WS::into_parser().ok().parse(ctx)? {
            let mut content = ctx.span();
            content.len = 0;

            while let Some(chars) = take_till(|c| c == '?').parse(ctx)? {
                content.extend_to_inclusive(chars);

                let qmarks = take_while(|c| c == '?')
                    .parse(ctx)?
                    .expect("at least one dash `-`");

                assert!(qmarks.len() > 0);

                content = content.extend_to_inclusive(qmarks);
                let (next, _) = ctx.peek();

                match next {
                    Some('>') => {
                        content.len -= 1;
                        ctx.next();

                        return Ok(Self {
                            target,
                            unparsed: Some(content),
                        });
                    }
                    _ => {}
                }
            }

            if content.len() == 0 {
                return Err(ControlFlow::Fatal(ReadError::PI(
                    ReadKind::PIUnparsed,
                    content,
                )));
            }
        }

        ensure_keyword("?>")
            .map_err(|_: ReadError| ReadError::PI(ReadKind::Suffix("?>"), start))
            .parse(ctx)?;

        Ok(Self {
            target,
            unparsed: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use parserc::{ControlFlow, ParseContext, Span};

    use super::*;

    #[test]
    fn test_pi() {
        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello?>")),
            Ok(PI {
                target: Span::new(2, 5, 1, 3),
                unparsed: None
            })
        );

        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello world? > ?>")),
            Ok(PI {
                target: Span::new(2, 5, 1, 3),
                unparsed: Some(Span::new(8, 9, 1, 9))
            })
        );

        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello ?>")),
            Err(ControlFlow::Fatal(ReadError::PI(
                ReadKind::PIUnparsed,
                Span::new(8, 0, 1, 9)
            )))
        );
    }
}
