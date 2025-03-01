use parserc::ParseContext;

use super::{ReadError, ReadEvent};

/// Read and parse a xml document.
///
/// Processing BOM, is not the responsibility of this `fn`.
///
/// On error, returns a [`ReadError`].
pub fn read_xml<'a, D>(doc: D) -> Result<Vec<ReadEvent>, ReadError>
where
    ParseContext<'a>: From<D>,
{
    let mut _ctx: ParseContext<'a> = doc.into();

    todo!()
}
