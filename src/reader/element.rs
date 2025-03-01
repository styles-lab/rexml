use parserc::{
    ControlFlow, FromSrc, IntoParser, Kind, ParseContext, Parser, ParserExt, ensure_char,
    ensure_keyword,
};

use crate::reader::{Attr, ReadKind, WS};

use super::{Name, ReadError, ReadEvent};

#[allow(unused)]
pub(super) fn parse_element_start(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<ReadEvent, ReadError> {
    let span = ctx.span();
    ensure_char('<')
        .map_err(|_: Kind| ReadError::Element(ReadKind::Prefix("<"), span))
        .parse(ctx)?;

    let name = Name::parse(ctx)?;

    WS::into_parser()
        .ok()
        .fatal(ReadError::Element(ReadKind::WS, ctx.span()))
        .parse(ctx)?;

    let mut attrs = vec![];

    while let Some(attr) = Attr::into_parser().ok().parse(ctx)? {
        attrs.push(attr);

        WS::into_parser()
            .ok()
            .fatal(ReadError::Element(ReadKind::WS, ctx.span()))
            .parse(ctx)?;
    }

    WS::into_parser()
        .ok()
        .fatal(ReadError::Element(ReadKind::WS, ctx.span()))
        .parse(ctx)?;

    if let Some(_) = ensure_keyword(">").ok().parse(ctx)? {
        return Ok(ReadEvent::ElementStart { name, attrs });
    }

    if let Some(_) = ensure_keyword("/>").ok().parse(ctx)? {
        return Ok(ReadEvent::EmptyElement { name, attrs });
    }

    Err(ControlFlow::Fatal(ReadError::Element(
        ReadKind::Suffix("`>` or `/>`"),
        ctx.span(),
    )))
}

#[allow(unused)]
pub(super) fn parse_element(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<Vec<ReadEvent>, ReadError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::{Attr, Name, ReadEvent, element::parse_element_start};

    #[test]
    fn test_el() {
        assert_eq!(
            parse_element_start(&mut ParseContext::from(
                r#"<termdef id="dt-dog" term="dog">"#
            )),
            Ok(ReadEvent::ElementStart {
                name: Name {
                    prefix: None,
                    local_name: Span::new(1, 7, 1, 2)
                },
                attrs: vec![
                    Attr {
                        name: Name {
                            prefix: None,
                            local_name: Span::new(9, 2, 1, 10)
                        },
                        value: Span::new(13, 6, 1, 14)
                    },
                    Attr {
                        name: Name {
                            prefix: None,
                            local_name: Span::new(21, 4, 1, 22)
                        },
                        value: Span::new(27, 3, 1, 28)
                    }
                ]
            })
        );

        assert_eq!(
            parse_element_start(&mut ParseContext::from(
                r#"<termdef id="dt-dog" term="dog" />"#
            )),
            Ok(ReadEvent::EmptyElement {
                name: Name {
                    prefix: None,
                    local_name: Span::new(1, 7, 1, 2)
                },
                attrs: vec![
                    Attr {
                        name: Name {
                            prefix: None,
                            local_name: Span::new(9, 2, 1, 10)
                        },
                        value: Span::new(13, 6, 1, 14)
                    },
                    Attr {
                        name: Name {
                            prefix: None,
                            local_name: Span::new(21, 4, 1, 22)
                        },
                        value: Span::new(27, 3, 1, 28)
                    }
                ]
            })
        );
    }
}
