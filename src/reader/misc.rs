use parserc::{
    ControlFlow, ParseContext, Parser, ParserExt, Span, ensure_char, ensure_char_if,
    ensure_keyword, take_till, take_while,
};

use super::{ReadError, ReadKind};

pub(super) fn skip_ws(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let span = take_while(|c| c.is_whitespace())
        .parse(ctx)?
        .ok_or(ControlFlow::Recoverable(ReadError::Ws(ctx.span())))?;

    Ok(span)
}

pub(super) fn parse_eq(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    skip_ws.ok().parse(ctx)?;
    let span = ensure_char('=')
        .fatal(ReadError::Eq(ctx.span()))
        .parse(ctx)?;
    skip_ws.ok().parse(ctx)?;
    Ok(span)
}

#[allow(unused)]
pub(super) fn parse_comment(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let start = ctx.span();

    ensure_keyword("<!--")
        .map_err(|_: ReadError| ReadError::Comment(ReadKind::CommentStart, start))
        .parse(ctx)?;

    let mut content = ctx.span();
    content.len = 0;

    while let Some(chars) = take_till(|c| c == '-').parse(ctx)? {
        content = content.extend_to_inclusive(chars);

        let dashes = take_while(|c| c == '-')
            .parse(ctx)?
            .expect("at least one dash `-`");

        assert!(dashes.len() > 0);

        content = content.extend_to_inclusive(dashes);

        if dashes.len() > 1 {
            let (next, _) = ctx.peek();

            match next {
                Some('>') => {
                    content.len -= 2;
                    ctx.next();
                    return Ok(content);
                }
                _ => {}
            }
        }
    }

    Err(ControlFlow::Fatal(ReadError::Comment(
        ReadKind::CommentEnd,
        ctx.span(),
    )))
}

pub(super) fn parse_name(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let start = ctx.span();
    let start = ensure_char_if(|c| c.is_alphabetic() || c == ':' || c == '_')
        .map_err(|_: ReadError| ReadError::Name(ReadKind::NameStartChar, start))
        .parse(ctx)?;

    let chars = take_while(|c| c == ':' || c == '_' || c == '-' || c == '.' || c.is_alphanumeric())
        .parse(ctx)?;

    if let Some(chars) = chars {
        Ok(start.extend_to_inclusive(chars))
    } else {
        Ok(start)
    }
}

pub(super) fn quote<F>(f: F) -> impl Parser<Output = Span, Error = ReadError>
where
    F: Fn(char) -> Result<(), ReadError>,
{
    move |ctx: &mut ParseContext<'_>| {
        let start = ctx.span();
        let double_quote = ensure_char('\'')
            .map(|_| false)
            .or(ensure_char('"').map(|_| true))
            .map_err(|_: ReadError| ReadError::Quote(ReadKind::QuoteStart, start))
            .parse(ctx)?;

        let quote = if double_quote { '"' } else { '\'' };

        let start = ctx.span();

        loop {
            let (next, span) = ctx.next();

            if let Some(next) = next {
                if next == quote {
                    return Ok(start.extend_to(span));
                }

                f(next).map_err(|err| ControlFlow::Fatal(err))?;
            } else {
                return Err(ControlFlow::Fatal(ReadError::Quote(
                    ReadKind::QuoteEnd,
                    span,
                )));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            parse_comment(&mut ParseContext::from("<!-- hello--good----->")),
            Ok(Span::new(4, 15, 1, 5))
        );

        assert_eq!(
            parse_comment(&mut ParseContext::from("hello--good----->")),
            Err(ControlFlow::Recoverable(ReadError::Comment(
                ReadKind::CommentStart,
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            parse_comment(&mut ParseContext::from("<!-- hello--good--")),
            Err(ControlFlow::Fatal(ReadError::Comment(
                ReadKind::CommentEnd,
                Span::new(18, 0, 1, 19)
            )))
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(
            parse_name(&mut ParseContext::from(":hello")),
            Ok(Span::new(0, 6, 1, 1))
        );
        assert_eq!(
            parse_name(&mut ParseContext::from("world:hello")),
            Ok(Span::new(0, 11, 1, 1))
        );
        assert_eq!(
            parse_name(&mut ParseContext::from("start-name")),
            Ok(Span::new(0, 10, 1, 1))
        );
        assert_eq!(
            parse_name(&mut ParseContext::from("start-name:")),
            Ok(Span::new(0, 11, 1, 1))
        );
        assert_eq!(
            parse_name(&mut ParseContext::from("-world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Name(
                ReadKind::NameStartChar,
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            parse_name(&mut ParseContext::from("1world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Name(
                ReadKind::NameStartChar,
                Span::new(0, 1, 1, 1)
            )))
        );
    }
}
