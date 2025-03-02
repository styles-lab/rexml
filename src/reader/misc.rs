use parserc::{
    ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Span, ensure_char,
    ensure_char_if, ensure_keyword, take_till, take_while,
};

use super::{CData, CharData, Comment, Name, PI, ReadError, ReadEvent, ReadKind, WS};

impl FromSrc for WS {
    type Error = ReadError;
    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let span = take_while(|c| c.is_whitespace())
            .parse(ctx)?
            .ok_or(ControlFlow::Recoverable(ReadError::WS))?;

        Ok(Self(span))
    }
}

pub(super) fn parse_eq(ctx: &mut ParseContext<'_>) -> parserc::Result<Span, ReadError> {
    WS::into_parser().ok().parse(ctx)?;
    let span = ensure_char('=').parse(ctx)?;
    WS::into_parser().ok().parse(ctx)?;
    Ok(span)
}

pub(super) fn quote<F>(f: F) -> impl Parser<Output = Span, Error = ReadError>
where
    F: Fn(char) -> Result<(), ReadError>,
{
    move |ctx: &mut ParseContext<'_>| {
        let double_quote = ensure_char('\'')
            .map(|_| false)
            .or(ensure_char('"').map(|_| true))
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
                    ReadKind::Suffix(if double_quote { "\"" } else { "'" }),
                    span,
                )));
            }
        }
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
        let start_char =
            ensure_char_if(|c| c == '_' || c == ':' || c.is_alphabetic()).parse(ctx)?;

        let suffix =
            take_while(|c| c == '_' || c == ':' || c == '-' || c == '.' || c.is_alphanumeric())
                .parse(ctx)?;

        if let Some(suffix) = suffix {
            Ok(Name(start_char.extend_to_inclusive(suffix)))
        } else {
            Ok(Name(start_char))
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
            .ok_or(ControlFlow::Recoverable(ReadError::CharData(
                ReadKind::None,
                ctx.span(),
            )))?;

        Ok(Self(span))
    }
}

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

        let target = Name::into_parser()
            .map_err(|_| ReadError::PI(ReadKind::PITarget, span))
            .parse(ctx)?;

        // check reserved word `('X' | 'x') ('M' | 'm') ('L' | 'l')`
        if ctx.as_str(target.0).to_lowercase() == "xml" {
            return Err(ControlFlow::Fatal(ReadError::PI(
                ReadKind::Reserved,
                target.0,
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
                return Err(ControlFlow::Fatal(ReadError::PI(ReadKind::PIBody, content)));
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

pub(super) fn parse_misc(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    WS::into_parser()
        .map(|v| ReadEvent::WS(v))
        .or(Comment::into_parser().map(|v| ReadEvent::Comment(v)))
        .or(PI::into_parser().map(|v| ReadEvent::PI(v)))
        .parse(ctx)
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
            Ok(Name(Span::new(0, 6, 1, 1)))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("world:hello")),
            Ok(Name(Span::new(0, 11, 1, 1)))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("start-name")),
            Ok(Name(Span::new(0, 10, 1, 1)))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("start-name:")),
            Ok(Name(Span::new(0, 11, 1, 1)))
        );
        assert_eq!(
            Name::parse(&mut ParseContext::from("-world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Parserc(
                parserc::Kind::EnsureCharIf
            )))
        );

        assert_eq!(
            Name::parse(&mut ParseContext::from("1world:hello")),
            Err(ControlFlow::Recoverable(ReadError::Parserc(
                parserc::Kind::EnsureCharIf
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

    #[test]
    fn test_pi() {
        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello?>")),
            Ok(PI {
                target: Name(Span::new(2, 5, 1, 3)),
                unparsed: None
            })
        );

        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello world? > ?>")),
            Ok(PI {
                target: Name(Span::new(2, 5, 1, 3)),
                unparsed: Some(Span::new(8, 9, 1, 9))
            })
        );

        assert_eq!(
            PI::parse(&mut ParseContext::from("<?hello ?>")),
            Err(ControlFlow::Fatal(ReadError::PI(
                ReadKind::PIBody,
                Span::new(8, 0, 1, 9)
            )))
        );
    }
}
