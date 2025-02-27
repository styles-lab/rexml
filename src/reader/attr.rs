use parserc::{ParseContext, Span};

use super::{ReadError, misc::Ref};

pub(super) enum AttrValue {
    Ref(Ref),
    Text(Span),
}

pub(super) fn parse_attr_value(
    ctx: &mut ParseContext<'_>,
) -> parserc::Result<Vec<AttrValue>, ReadError> {
    todo!()
}
