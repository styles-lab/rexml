use parserc::{
    ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Span, ensure_char,
    ensure_char_if, ensure_keyword, take_till, take_while,
};

use super::{CData, CharData, Comment, Name, PI, ReadError, ReadEvent, ReadKind, WS};

pub(super) fn parse_eq(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    WS::into_parser().ok().parse(ctx)?;
    let span = ensure_char('=')
        .fatal(ReadError::Eq(ctx.span()))
        .parse(ctx)?;
    WS::into_parser().ok().parse(ctx)?;
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

impl FromSrc for WS {
    type Error = ReadError;
    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let span = take_while(|c| c.is_whitespace())
            .parse(ctx)?
            .ok_or(ControlFlow::Recoverable(ReadError::Ws(ctx.span())))?;

        Ok(Self(span))
    }
}

impl FromSrc for Comment {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
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
                        return Ok(Self(content));
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
}

impl FromSrc for Name {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();
        let start_char = ensure_char_if(|c| c.is_alphabetic() || c == '_')
            .map_err(|_: ReadError| ReadError::Name(ReadKind::NameStartChar, start))
            .parse(ctx)?;

        let prefix =
            take_while(|c| c == '_' || c == '-' || c == '.' || c.is_alphanumeric()).parse(ctx)?;

        let prefix = if let Some(prefix) = prefix {
            start_char.extend_to_inclusive(prefix)
        } else {
            start_char
        };

        if let Some(_) = ensure_char(':').ok().parse(ctx)? {
            let local_name =
                take_while(|c| c == '_' || c == '-' || c == '.' || c.is_alphanumeric())
                    .parse(ctx)?
                    .ok_or(ControlFlow::Fatal(ReadError::Name(
                        ReadKind::LocalName,
                        ctx.span(),
                    )))?;

            Ok(Self {
                prefix: Some(prefix),
                local_name,
            })
        } else {
            Ok(Self {
                prefix: None,
                local_name: prefix,
            })
        }
    }
}

impl FromSrc for CData {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
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
                let (next, _) = ctx.peek();

                if let Some(next) = next {
                    match next {
                        '>' => {
                            content.len -= 2;

                            ctx.next();
                            return Ok(Self(content));
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
}

impl FromSrc for CharData {
    type Error = ReadError;

    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let span = take_till(|c| c == '<')
            .parse(ctx)?
            .ok_or(ControlFlow::Recoverable(ReadError::CharData))?;

        Ok(Self(span))
    }
}

pub(super) fn parse_misc(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    WS::into_parser()
        .map(|v| ReadEvent::WS(v))
        .or(Comment::into_parser().map(|v| ReadEvent::Comment(v)))
        .or(PI::into_parser().map(|v| ReadEvent::PI(v)))
        .parse(ctx)
}

#[allow(unused)]
pub(super) fn parse_miscs(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<Vec<ReadEvent>, ReadError> {
    let mut events = vec![];
    while let Some(event) = parse_misc.ok().parse(ctx)? {
        events.push(event);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unparsed_chardata() {
        assert_eq!(
            CharData::parse(&mut ParseContext::from("111111\n111"),),
            Ok(CharData(Span::new(0, 10, 1, 1)))
        );

        assert_eq!(
            CharData::parse(&mut ParseContext::from("\n111111111<"),),
            Ok(CharData(Span::new(0, 10, 1, 1)))
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            Comment::parse(&mut ParseContext::from("<!------->")),
            Ok(Comment(Span::new(4, 3, 1, 5)))
        );
        assert_eq!(
            Comment::parse(&mut ParseContext::from("<!-- hello--good----->")),
            Ok(Comment(Span::new(4, 15, 1, 5)))
        );

        assert_eq!(
            Comment::parse(&mut ParseContext::from("hello--good----->")),
            Err(ControlFlow::Recoverable(ReadError::Comment(
                ReadKind::Prefix("<!--"),
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            Comment::parse(&mut ParseContext::from("<!-- hello--good--")),
            Err(ControlFlow::Fatal(ReadError::Comment(
                ReadKind::Suffix("-->"),
                Span::new(18, 0, 1, 19)
            )))
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(
            Name::parse(&mut ParseContext::from(":hello")),
            Err(ControlFlow::Recoverable(ReadError::Name(
                ReadKind::NameStartChar,
                Span {
                    offset: 0,
                    len: 1,
                    lines: 1,
                    cols: 1
                }
            )))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("world:hello")),
            Ok(Name {
                prefix: Some(Span::new(0, 5, 1, 1)),
                local_name: Span::new(6, 5, 1, 7)
            })
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("start-name")),
            Ok(Name {
                prefix: None,
                local_name: Span::new(0, 10, 1, 1)
            })
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("start-name:")),
            Err(ControlFlow::Fatal(ReadError::Name(
                ReadKind::LocalName,
                Span {
                    offset: 11,
                    len: 0,
                    lines: 1,
                    cols: 12
                }
            )))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("-world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Name(
                ReadKind::NameStartChar,
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            Name::parse(&mut ParseContext::from("1world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Name(
                ReadKind::NameStartChar,
                Span::new(0, 1, 1, 1)
            )))
        );
    }

    #[test]
    fn test_cdata() {
        assert_eq!(
            CData::parse(&mut ParseContext::from("<![CDATA[]]he;<>]]>")),
            Ok(CData(Span::new(9, 7, 1, 10)))
        );

        assert_eq!(
            CData::parse(&mut ParseContext::from("]]he;<>]]>")),
            Err(ControlFlow::Recoverable(ReadError::CData(
                ReadKind::Prefix("<![CDATA["),
                Span::new(0, 1, 1, 1)
            )))
        );

        assert_eq!(
            CData::parse(&mut ParseContext::from("<![CDATA[]]he;<>")),
            Err(ControlFlow::Fatal(ReadError::CData(
                ReadKind::Suffix("]]>"),
                Span::new(16, 0, 1, 17)
            )))
        );
    }
}
