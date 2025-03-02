use parserc::{FromSpan, FromSrc, IntoParser, ParseContext, Parser, ParserExt};

use crate::reader::{
    Name, ReadKind, WS,
    misc::{parse_eq, quote},
};

use super::{Attr, ReadError, Start};

impl Start {
    /// Create an attribute list iterator [`Attrs`].
    pub fn attrs<'a, S>(&self, source: S) -> Attrs<'a>
    where
        S: FromSpan<'a>,
    {
        Attrs {
            ctx: source.from_span(self.attrs).into(),
        }
    }
}

fn parse_attr(ctx: &mut ParseContext<'_>) -> parserc::Result<Attr, ReadError> {
    WS::into_parser().ok().parse(ctx)?;

    let name = Name::parse(ctx)?;

    parse_eq
        .fatal(ReadError::Attr(ReadKind::Eq, ctx.span()))
        .parse(ctx)?;

    let value = quote(|_| Ok(()))
        .fatal(ReadError::Attr(ReadKind::Quote, ctx.span()))
        .parse(ctx)?;

    Ok(Attr { name, value })
}

/// A iterator over attribute list.
pub struct Attrs<'a> {
    ctx: ParseContext<'a>,
}

impl<'a> Iterator for Attrs<'a> {
    type Item = Result<Attr, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        match parse_attr.ok().parse(&mut self.ctx) {
            Ok(Some(attr)) => Some(Ok(attr)),
            Ok(None) => None,
            Err(err) => Some(Err(err.into_raw())),
        }
    }
}

#[cfg(test)]
mod tests {
    use parserc::{ParseContext, Span};

    use crate::reader::{Attr, Name};

    use super::Attrs;

    #[test]
    fn test_attrs() {
        assert_eq!(
            Attrs {
                ctx: ParseContext::from(
                    r#"
                    id="dt-dog" 
                    
                    g:term="dog"
                    "#
                ),
            }
            .collect::<Result<Vec<_>, _>>(),
            Ok(vec![
                Attr {
                    name: Name(Span::new(21, 2, 2, 21)),
                    value: Span::new(25, 6, 2, 25)
                },
                Attr {
                    name: Name(Span::new(75, 6, 4, 21)),
                    value: Span::new(83, 3, 4, 29)
                }
            ])
        );
    }
}
