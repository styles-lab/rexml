use parserc::{ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Span};

use crate::reader::{
    DocType, XmlDecl,
    element::{parse_content, parse_element_empty_or_start},
    misc::parse_misc,
};

use super::{ReadError, ReadEvent};

#[allow(unused)]
enum ReadState {
    Init,
    XmlDecl,
    XmlDeclMiscs,
    DocType,
    DocTypeMiscs,
    RootElement,
    Element,
    ElementMiscs,
    Eof,
}

/// An iterator/parser of xml document sections.
#[allow(unused)]
pub struct Reader<'a> {
    ctx: ParseContext<'a>,
    state: ReadState,
    elem_starts: Vec<Span>,
}

impl<'a> Reader<'a> {
    pub fn new<S>(ctx: S) -> Self
    where
        ParseContext<'a>: From<S>,
    {
        Self {
            ctx: ctx.into(),
            state: ReadState::Init,
            elem_starts: vec![],
        }
    }

    fn handle_elem_event(&mut self, event: &ReadEvent) -> Result<(), ReadError> {
        match &event {
            ReadEvent::ElemStart(start) => {
                self.elem_starts.push(start.name.0);
            }
            ReadEvent::ElemEnd(end) => {
                if let Some(start_tag) = self.elem_starts.pop() {
                    if self.ctx.as_str(start_tag) != self.ctx.as_str(end.0.0) {
                        return Err(ReadError::ElemTagMismatch(start_tag, end.0.0));
                    }
                } else {
                    return Err(ReadError::MissStartTag(end.0.0));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<ReadEvent, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                ReadState::Init => {
                    self.state = ReadState::XmlDecl;
                    continue;
                }
                ReadState::XmlDecl => {
                    self.state = ReadState::XmlDeclMiscs;

                    return Some(
                        XmlDecl::parse(&mut self.ctx)
                            .map_err(ControlFlow::into_raw)
                            .map(|v| ReadEvent::XmlDecl(v)),
                    );
                }
                ReadState::XmlDeclMiscs => {
                    match parse_misc
                        .ok()
                        .parse(&mut self.ctx)
                        .map_err(ControlFlow::into_raw)
                    {
                        Ok(Some(v)) => return Some(Ok(v)),
                        Ok(None) => {
                            self.state = ReadState::DocType;
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::DocType => {
                    match DocType::into_parser()
                        .ok()
                        .parse(&mut self.ctx)
                        .map_err(ControlFlow::into_raw)
                    {
                        Ok(Some(doctype)) => {
                            self.state = ReadState::DocTypeMiscs;
                            return Some(Ok(ReadEvent::DocType(doctype)));
                        }
                        Ok(None) => {
                            self.state = ReadState::RootElement;
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::DocTypeMiscs => {
                    match parse_misc
                        .ok()
                        .parse(&mut self.ctx)
                        .map_err(ControlFlow::into_raw)
                    {
                        Ok(Some(v)) => return Some(Ok(v)),
                        Ok(None) => {
                            self.state = ReadState::RootElement;
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::RootElement => {
                    match parse_element_empty_or_start(&mut self.ctx).map_err(ControlFlow::into_raw)
                    {
                        Ok(event) => {
                            if let Err(err) = self.handle_elem_event(&event) {
                                return Some(Err(err));
                            }

                            if self.elem_starts.is_empty() {
                                self.state = ReadState::ElementMiscs;
                            } else {
                                self.state = ReadState::Element;
                            }

                            return Some(Ok(event));
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::Element => {
                    match parse_content(&mut self.ctx).map_err(ControlFlow::into_raw) {
                        Ok(event) => {
                            if let Err(err) = self.handle_elem_event(&event) {
                                return Some(Err(err));
                            }

                            if self.elem_starts.is_empty() {
                                self.state = ReadState::ElementMiscs;
                            }

                            return Some(Ok(event));
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::ElementMiscs => {
                    match parse_misc
                        .ok()
                        .parse(&mut self.ctx)
                        .map_err(ControlFlow::into_raw)
                    {
                        Ok(Some(v)) => return Some(Ok(v)),
                        Ok(None) => {
                            self.state = ReadState::Eof;
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
                ReadState::Eof => return None,
            }
        }
    }
}
