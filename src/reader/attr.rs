use parserc::{FromSrc, ParseContext, Parser, ParserExt};

use crate::reader::{
    Name, ReadKind,
    misc::{parse_eq, quote},
};

use super::{Attr, ReadError};

impl FromSrc for Attr {
    type Error = ReadError;
    fn parse(ctx: &mut ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let name = Name::parse(ctx)?;
        parse_eq
            .fatal(ReadError::Attr(ReadKind::Eq, ctx.span()))
            .parse(ctx)?;

        let value = quote(|_| Ok(()))
            .fatal(ReadError::Attr(ReadKind::Quote, ctx.span()))
            .parse(ctx)?;

        Ok(Attr { name, value })
    }
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext, Span};

    use crate::reader::{Attr, Name};

    #[test]
    fn test_attr() {
        assert_eq!(
            Attr::parse(&mut ParseContext::from(r#"color="rgb(255,255,255)""#),),
            Ok(Attr {
                name: Name {
                    prefix: None,
                    local_name: Span::new(0, 5, 1, 1)
                },
                value: Span::new(7, 16, 1, 8),
            })
        );

        assert_eq!(
            Attr::parse(&mut ParseContext::from(
                r#"color 
                    = "rgb(255,255,255)"
                "#
            ),),
            Ok(Attr {
                name: Name {
                    prefix: None,
                    local_name: Span::new(0, 5, 1, 1)
                },
                value: Span::new(30, 16, 2, 24),
            })
        );
    }
}
