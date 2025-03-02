use parserc::{FromSpan, ParseContext};

use super::Start;

impl Start {
    /// Create an attribute list iterator [`Attrs`].
    pub fn attrs<'a, S>(&self, ctx: S) -> Attrs<'a>
    where
        S: FromSpan<'a>,
    {
        Attrs {
            ctx: ctx.from_span(self.attrs).into(),
        }
    }
}

/// A iterator over attribute list.
#[allow(unused)]
pub struct Attrs<'a> {
    ctx: ParseContext<'a>,
}
