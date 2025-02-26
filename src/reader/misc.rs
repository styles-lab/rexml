use parserc::{
    ControlFlow, FromSrc, ParseContext, Parser, ParserExt, Span, ensure_char, ensure_char_if,
    ensure_keyword, take_till, take_while,
};

use super::{ReadError, ReadKind, pi::parse_pi};

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

pub(super) fn quote<F>(f: F) -> impl Parser<Output = Span, Error = ReadError>
where
    F: Fn(char) -> Result<(), ReadError>,
{
    move |ctx: &mut ParseContext<'_>| {
        let start = ctx.span();
        let double_quote = ensure_char('\'')
            .map(|_| false)
            .or(ensure_char('"').map(|_| true))
            .map_err(|_: ReadError| ReadError::Quote(ReadKind::Prefix("`'` or `\"`"), start))
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
                    ReadKind::Suffix("`'` or `\"`"),
                    span,
                )));
            }
        }
    }
}

pub(super) fn parse_comment(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let start = ctx.span();

    ensure_keyword("<!--")
        .map_err(|_: ReadError| ReadError::Comment(ReadKind::Prefix("<!--"), start))
        .parse(ctx)?;

    let mut content = ctx.span();
    content.len = 0;

    loop {
        if let Some(chars) = take_till(|c| c == '-').parse(ctx)? {
            content = content.extend_to_inclusive(chars);
        }

        let dashes = match take_while(|c| c == '-').parse(ctx)? {
            Some(dashes) => dashes,
            _ => {
                break;
            }
        };

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
        ReadKind::Suffix("-->"),
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

pub(super) enum CharRef {
    Decimal(Span),
    Hex(Span),
}

pub(super) enum Ref {
    Entity(Span),
    Char(CharRef),
}

pub(super) fn parse_char_ref(ctx: &mut ParseContext<'_>) -> parserc::Result<CharRef, ReadError> {
    let start = ctx.span();
    let hex = ensure_keyword("&#")
        .map(|_| false)
        .or(ensure_keyword("&#x").map(|_| true))
        .map_err(|_: ReadError| ReadError::CharRef(ReadKind::Prefix("&# or &#x"), start))
        .parse(ctx)?;

    let value = if hex {
        take_while(|c| c.is_ascii_hexdigit()).parse(ctx)?
    } else {
        take_while(|c| c.is_ascii_digit()).parse(ctx)?
    };

    let value = value.ok_or(ControlFlow::Fatal(ReadError::CharRef(
        ReadKind::LitNum,
        ctx.span(),
    )))?;

    if hex {
        Ok(CharRef::Hex(value))
    } else {
        Ok(CharRef::Decimal(value))
    }
}

pub(super) fn parse_entity_ref(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let start = ctx.span();
    ensure_char('&')
        .map_err(|_: ReadError| ReadError::EntityRef(ReadKind::Prefix("%"), start))
        .parse(ctx)?;

    let name = parse_name
        .fatal(ReadError::EntityRef(ReadKind::Name, ctx.span()))
        .parse(ctx)?;

    ensure_char(';')
        .fatal(ReadError::EntityRef(ReadKind::Suffix(";"), ctx.span()))
        .parse(ctx)?;

    Ok(name)
}

pub(super) fn parse_ref(ctx: &mut ParseContext<'_>) -> parserc::Result<Ref, ReadError> {
    parse_entity_ref
        .map(|v| Ref::Entity(v))
        .or(parse_char_ref.map(|v| Ref::Char(v)))
        .parse(ctx)
}

pub(super) enum Misc {
    Comment(Span),
    PI { name: Span, unparsed: Option<Span> },
    WS(Span),
}

pub(super) fn parse_misc(ctx: &mut ParseContext<'_>) -> parserc::Result<Misc, ReadError> {
    parse_comment
        .map(|span| Misc::Comment(span))
        .or(skip_ws.map(|span| Misc::WS(span)))
        .or(parse_pi.map(|(name, unparsed)| Misc::PI { name, unparsed }))
        .parse(ctx)
}

pub(super) fn parse_cdata(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    let span = ctx.span();
    ensure_keyword("<![CDATA[")
        .map_err(|_: ReadError| ReadError::CData(ReadKind::Prefix("<![CDATA["), span))
        .parse(ctx)?;

    let mut content = ctx.span();
    content.len = 0;

    loop {
        if let Some(chars) = take_till(|c| c == ']').parse(ctx)? {
            content = content.extend_to_inclusive(chars);
        }

        let chars = match take_while(|c| c == ']').parse(ctx)? {
            Some(chars) => chars,
            _ => break,
        };

        content = content.extend_to_inclusive(chars);

        if chars.len() > 1 {
            let (next, span) = ctx.peek();

            if let Some(next) = next {
                match next {
                    '>' => {
                        content.len -= 2;

                        ctx.next();
                        return Ok(content);
                    }
                    _ => {}
                }
            }
        }
    }

    Err(ControlFlow::Fatal(ReadError::CData(
        ReadKind::Suffix("]]>"),
        ctx.span(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            parse_comment(&mut ParseContext::from("<!------->")),
            Ok(Span::new(4, 3, 1, 5))
        );
        assert_eq!(
            parse_comment(&mut ParseContext::from("<!-- hello--good----->")),
            Ok(Span::new(4, 15, 1, 5))
        );

        assert_eq!(
            parse_comment(&mut ParseContext::from("hello--good----->")),
            Err(ControlFlow::Recoverable(ReadError::Comment(
                ReadKind::Prefix("<!--"),
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            parse_comment(&mut ParseContext::from("<!-- hello--good--")),
            Err(ControlFlow::Fatal(ReadError::Comment(
                ReadKind::Suffix("-->"),
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

    #[test]
    fn test_cdata() {
        assert_eq!(
            parse_cdata(&mut ParseContext::from("<![CDATA[]]he;<>]]>")),
            Ok(Span::new(9, 7, 1, 10))
        );

        assert_eq!(
            parse_cdata(&mut ParseContext::from("]]he;<>]]>")),
            Err(ControlFlow::Recoverable(ReadError::CData(
                ReadKind::Prefix("<![CDATA["),
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            parse_cdata(&mut ParseContext::from("<![CDATA[]]he;<>")),
            Err(ControlFlow::Fatal(ReadError::CData(
                ReadKind::Suffix("]]>"),
                Span::new(16, 0, 1, 17)
            )))
        );
    }
}
