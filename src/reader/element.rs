use parserc::{
    ControlFlow, IntoParser, Kind, ParseContext, Parser, ParserExt, ensure_char, ensure_keyword,
    take_till,
};

use super::{
    CData, CharData, Comment, End, Name, PI, ReadError, ReadEvent, ReadKind, Start, WS, misc::quote,
};

pub(super) fn parse_element_empty_or_start(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<ReadEvent, ReadError> {
    let span = ctx.span();
    ensure_char('<')
        .map_err(|_: Kind| ReadError::StartTag(ReadKind::Prefix("<"), span))
        .parse(ctx)?;

    let name = Name::into_parser()
        .fatal(ReadError::StartTag(ReadKind::TagName, ctx.span()))
        .parse(ctx)?;

    let mut attrs = ctx.span();
    attrs.len = 0;

    while let Some(span) =
        take_till(|c| c == '"' || c == '\'' || c == '>' || c == '/').parse(ctx)?
    {
        attrs = attrs.extend_to_inclusive(span);
        let (next, _) = ctx.peek();

        if let Some(next) = next {
            match next {
                '\'' | '"' => {
                    let _ = quote(|_| Ok(())).parse(ctx)?;
                    attrs = attrs.extend_to(ctx.span());
                    continue;
                }
                _ => {}
            }
        }

        break;
    }

    WS::into_parser().ok().parse(ctx)?;

    if let Some(_) = ensure_keyword(">").ok().parse(ctx)? {
        return Ok(ReadEvent::ElemStart(Start { name, attrs }));
    }

    if let Some(_) = ensure_keyword("/>").ok().parse(ctx)? {
        return Ok(ReadEvent::EmptyElem(Start { name, attrs }));
    }

    Err(ControlFlow::Fatal(ReadError::StartTag(
        ReadKind::Suffix("`>` or `/>`"),
        ctx.span(),
    )))
}

fn parse_element_end(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    let span = ctx.span();

    ensure_keyword("</")
        .map_err(|_: Kind| ReadError::EndTag(ReadKind::Prefix("</"), span))
        .parse(ctx)?;

    let name = Name::into_parser()
        .fatal(ReadError::EndTag(ReadKind::TagName, span))
        .parse(ctx)?;

    WS::into_parser().ok().parse(ctx)?;

    ensure_char('>')
        .map_err(|_: Kind| ReadError::EndTag(ReadKind::Suffix(">"), span))
        .parse(ctx)?;

    Ok(ReadEvent::ElemEnd(End(name)))
}

pub(super) fn parse_content(ctx: &mut ParseContext<'_>) -> parserc::Result<ReadEvent, ReadError> {
    parse_element_end
        .or(PI::into_parser().map(|c| ReadEvent::PI(c)))
        .or(Comment::into_parser().map(|c| ReadEvent::Comment(c)))
        .or(parse_element_empty_or_start)
        .or(CharData::into_parser().map(|c| ReadEvent::CharData(c)))
        .or(CData::into_parser().map(|c| ReadEvent::CData(c)))
        .parse(ctx)
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::{Name, ReadEvent, Start, element::parse_element_empty_or_start};

    #[test]
    fn test_el_empty_or_start() {
        assert_eq!(
            parse_element_empty_or_start(&mut ParseContext::from(
                r#"<termdef id="dt-dog" term="dog">"#
            )),
            Ok(ReadEvent::ElemStart(Start {
                name: Name(Span::new(1, 7, 1, 2)),
                attrs: Span::new(8, 23, 1, 9)
            }))
        );

        assert_eq!(
            parse_element_empty_or_start(&mut ParseContext::from(
                r#"<termdef id="dt-dog" term="dog" />"#
            )),
            Ok(ReadEvent::EmptyElem(Start {
                name: Name(Span::new(1, 7, 1, 2)),
                attrs: Span::new(8, 24, 1, 9)
            }))
        );
    }
}
