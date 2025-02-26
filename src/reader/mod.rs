mod errors;
pub use errors::*;

mod doctype;
mod misc;
mod pi;
mod xmldecl;

use crate::events::Event;
use parserc::ParseContext;

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

    xmldecl::parse_xml_decl(&mut ctx).map_err(|err| err.into_raw())?;

    todo!()
}
