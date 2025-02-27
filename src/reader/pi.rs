use parserc::{
    ControlFlow, ParseContext, Parser, ParserExt, Span, ensure_keyword, take_till, take_while,
};

use super::{
    ReadError, ReadKind,
    misc::{parse_name, skip_ws},
};

pub(super) fn parse_pi(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<(Span, Option<Span>), ReadError> {
    let start = ctx.span();
    ensure_keyword("<?")
        .map_err(|_: ReadError| ReadError::PI(ReadKind::Prefix("<?"), start))
        .parse(ctx)?;

    let span = ctx.span();

    let name = parse_name
        .map_err(|_| ReadError::PI(ReadKind::PITarget, span))
        .parse(ctx)?;

    // check reserved word `('X' | 'x') ('M' | 'm') ('L' | 'l')`
    if ctx.as_str(name).to_lowercase() == "xml" {
        return Err(ControlFlow::Fatal(ReadError::PI(
            ReadKind::ReservedXml,
            name,
        )));
    }

    if let Some(_) = skip_ws.ok().parse(ctx)? {
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
                    return Ok((name, Some(content)));
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

    Ok((name, None))
}

#[cfg(test)]
mod tests {
    use parserc::{ControlFlow, ParseContext, Span};

    use super::*;

    #[test]
    fn test_pi() {
        assert_eq!(
            parse_pi(&mut ParseContext::from("<?hello?>")),
            Ok((Span::new(2, 5, 1, 3), None))
        );

        assert_eq!(
            parse_pi(&mut ParseContext::from("<?hello world? > ?>")),
            Ok((Span::new(2, 5, 1, 3), Some(Span::new(8, 9, 1, 9))))
        );

        assert_eq!(
            parse_pi(&mut ParseContext::from("<?hello ?>")),
            Err(ControlFlow::Fatal(ReadError::PI(
                ReadKind::PIUnparsed,
                Span::new(8, 0, 1, 9)
            )))
        );
    }
}
