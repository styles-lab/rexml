use parserc::{ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt};

use crate::reader::{DocType, XmlDecl, element::parse_element};

use super::{ReadError, ReadEvent, misc::parse_misc};

/// Read and parse a xml document.
///
/// Processing BOM, is not the responsibility of this `fn`.
///
/// On error, returns a [`ReadError`].
pub fn read_xml<'a, D>(doc: D) -> Result<Vec<ReadEvent>, ReadError>
where
    ParseContext<'a>: From<D>,
{
    let mut ctx: ParseContext<'a> = doc.into();

    let mut events = vec![ReadEvent::XmlDecl(
        XmlDecl::parse(&mut ctx).map_err(ControlFlow::into_raw)?,
    )];

    // let mut miscs = parse_miscs(&mut ctx).map_err(ControlFlow::into_raw)?;

    // events.append(&mut miscs);

    while let Some(event) = parse_misc
        .ok()
        .parse(&mut ctx)
        .map_err(ControlFlow::into_raw)?
    {
        events.push(event);
    }

    if let Some(doctype) = DocType::into_parser()
        .ok()
        .parse(&mut ctx)
        .map_err(ControlFlow::into_raw)?
    {
        events.push(ReadEvent::DocType(doctype));

        // let mut miscs = parse_miscs(&mut ctx).map_err(ControlFlow::into_raw)?;

        // events.append(&mut miscs);

        while let Some(event) = parse_misc
            .ok()
            .parse(&mut ctx)
            .map_err(ControlFlow::into_raw)?
        {
            events.push(event);
        }
    }

    let mut els = parse_element(&mut ctx).map_err(ControlFlow::into_raw)?;

    events.append(&mut els);

    // let mut miscs = parse_miscs(&mut ctx).map_err(ControlFlow::into_raw)?;

    // events.append(&mut miscs);

    while let Some(event) = parse_misc
        .ok()
        .parse(&mut ctx)
        .map_err(ControlFlow::into_raw)?
    {
        events.push(event);
    }

    Ok(events)
}
