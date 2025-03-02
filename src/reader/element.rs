use parserc::{
    ControlFlow, FromSrc, IntoParser, Kind, ParseContext, Parser, ParserExt, ensure_char,
    ensure_keyword,
};

use crate::reader::{Attr, CData, CharData, Comment, PI, ReadKind, WS};

use super::{Name, ReadError, ReadEvent};

#[allow(unused)]
pub(super) fn parse_element_empty_or_start(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<ReadEvent, ReadError> {
    let span = ctx.span();
    ensure_char('<')
        .map_err(|_: Kind| ReadError::Element(ReadKind::Prefix("<"), span))
        .parse(ctx)?;

    let name = Name::parse(ctx)?;

    WS::into_parser().ok().parse(ctx)?;

    let mut attrs = vec![];

    while let Some(attr) = Attr::into_parser().ok().parse(ctx)? {
        attrs.push(attr);

        WS::into_parser().ok().parse(ctx)?;
    }

    WS::into_parser().ok().parse(ctx)?;

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

pub fn parse_element_end(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    let span = ctx.span();
    ensure_keyword("</")
        .map_err(|_: Kind| ReadError::Element(ReadKind::Prefix("</"), span))
        .parse(ctx)?;

    let name = Name::into_parser()
        .fatal(ReadError::Element(ReadKind::Name, span))
        .parse(ctx)?;

    WS::into_parser().ok().parse(ctx)?;

    ensure_char('>')
        .map_err(|_: Kind| ReadError::Element(ReadKind::Suffix(">"), span))
        .parse(ctx)?;

    Ok(ReadEvent::ElementEnd(name))
}

fn parse_content(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    parse_element_empty_or_start
        .or(parse_element_end)
        .or(CharData::into_parser().map(|c| ReadEvent::CharData(c)))
        .or(CData::into_parser().map(|c| ReadEvent::CData(c)))
        .or(PI::into_parser().map(|c| ReadEvent::PI(c)))
        .or(Comment::into_parser().map(|c| ReadEvent::Comment(c)))
        .parse(ctx)
}

#[allow(unused)]
pub(super) fn parse_element(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<Vec<ReadEvent>, ReadError> {
    let mut events = vec![];

    let mut event = parse_element_empty_or_start(ctx)?;

    let mut elem_starts = vec![];

    loop {
        match &event {
            ReadEvent::ElementStart { name, attrs: _ } => {
                elem_starts.push(*name);
            }
            ReadEvent::ElementEnd(name) => {
                if let Some(start_tag) = elem_starts.pop() {
                    if ctx.as_str(start_tag.local_name) != ctx.as_str(name.local_name) {
                        return Err(ControlFlow::Fatal(ReadError::Mismatch(start_tag, *name)));
                    }

                    if let Some(start_tag_prefix) = start_tag.prefix {
                        if let Some(prefix) = name.prefix {
                            if ctx.as_str(prefix) != ctx.as_str(start_tag_prefix) {
                                return Err(ControlFlow::Fatal(ReadError::Mismatch(
                                    start_tag, *name,
                                )));
                            }
                        } else {
                            return Err(ControlFlow::Fatal(ReadError::Mismatch(start_tag, *name)));
                        }
                    }
                } else {
                    return Err(ControlFlow::Fatal(ReadError::HangEndTag(*name)));
                }
            }
            _ => {}
        }

        events.push(event);

        if elem_starts.is_empty() {
            return Ok(events);
        }

        if let Some(e) = parse_content.ok().parse(ctx)? {
            event = e;
        } else {
            return Err(ControlFlow::Fatal(ReadError::Unclosed(
                elem_starts,
                ctx.span(),
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::{
        Attr, CData, CharData, Comment, Name, PI, ReadEvent, element::parse_element_empty_or_start,
    };

    use super::parse_element;

    #[test]
    fn test_el_empty_or_start() {
        assert_eq!(
            parse_element_empty_or_start(&mut ParseContext::from(
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
            parse_element_empty_or_start(&mut ParseContext::from(
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

    #[test]
    fn test_element() {
        assert_eq!(
            parse_element(&mut ParseContext::from("<hello />")),
            Ok(vec![ReadEvent::EmptyElement {
                name: Name {
                    prefix: None,
                    local_name: Span::new(1, 5, 1, 2)
                },
                attrs: vec![]
            }])
        );

        assert_eq!(
            parse_element(&mut ParseContext::from(
                r#"<g:hello>
                    hello world
                    <!---hello world-->
                    <?xxxx target?>
                    <![CDATA[ <<]]>
                   </g:hello> 
                "#
            )),
            Ok(vec![
                ReadEvent::ElementStart {
                    name: Name {
                        prefix: Some(Span::new(1, 1, 1, 2)),
                        local_name: Span::new(3, 5, 1, 4)
                    },
                    attrs: vec![]
                },
                ReadEvent::CharData(CharData(Span::new(9, 53, 1, 10))),
                ReadEvent::Comment(Comment(Span::new(66, 12, 3, 25))),
                ReadEvent::CharData(CharData(Span::new(81, 21, 3, 40))),
                ReadEvent::PI(PI {
                    target: Name {
                        prefix: None,
                        local_name: Span::new(104, 4, 4, 23)
                    },
                    unparsed: Some(Span::new(109, 6, 4, 28))
                }),
                ReadEvent::CharData(CharData(Span::new(117, 21, 4, 36))),
                ReadEvent::CData(CData(Span::new(147, 3, 5, 30))),
                ReadEvent::CharData(CharData(Span::new(153, 20, 5, 36))),
                ReadEvent::ElementEnd(Name {
                    prefix: Some(Span::new(175, 1, 6, 22)),
                    local_name: Span::new(177, 5, 6, 24)
                }),
            ])
        );
    }
}
