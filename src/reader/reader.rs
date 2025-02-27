use crate::{events::Event, reader::xmldecl::parse_xml_decl};
use parserc::ParseContext;

use super::ReadError;

/// Read and parse a xml document.
///
/// Processing BOM, is not the responsibility of this `fn`.
///
/// On error, returns a [`ReadError`].
pub fn read_xml<'a, D>(doc: D) -> Result<Vec<Event<'a>>, ReadError>
where
    ParseContext<'a>: From<D>,
{
    let mut ctx: ParseContext<'a> = doc.into();

    parse_xml_decl(&mut ctx).map_err(|err| err.into_raw())?;

    todo!()
}
