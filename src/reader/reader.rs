use std::fmt::Debug;

use parserc::{AsBytes, ControlFlow, Input, Parse, Parser, ParserExt};

use super::{
    CData, CharData, Comment, DocType, ElemEnd, ElemStart, PI, ReadError, XmlDecl, ensure_ws,
};

/// Xml node type returns by [`XmlReader`].
#[derive(Debug, PartialEq, Clone)]
pub enum XmlNode<I> {
    XmlDecl(XmlDecl<I>),
    DocType(DocType<I>),
    PI(PI<I>),
    /// Whitespace.
    S(I),
    Comment(Comment<I>),
    Start(ElemStart<I>),
    End(ElemEnd<I>),
    CharData(CharData<I>),
    CData(CData<I>),
}

/// State of reader.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReadState {
    XmlDecl,
    MiscBeforeDocType,
    DocType,
    MiscBeforeElement,
    RootElement,
    Element,
    MiscAfterElement,
    Eof,
}

/// Xml document reader.
pub struct XmlReader<I> {
    /// read state of this reader.
    state: ReadState,
    /// input stream.
    input: I,
    /// start tag counter.
    starts: usize,
}

impl<I> XmlReader<I>
where
    I: Input<Item = u8> + AsBytes + Clone + Debug,
{
    #[inline(always)]
    fn read_xml_decl(&mut self) -> Result<XmlNode<I>, ControlFlow<ReadError<I>>> {
        let (decl, input) = XmlDecl::parse(self.input.clone())?;

        self.input = input;

        self.state = ReadState::MiscBeforeDocType;

        Ok(XmlNode::XmlDecl(decl))
    }

    #[inline(always)]
    fn read_doctype(&mut self) -> Result<Option<XmlNode<I>>, ControlFlow<ReadError<I>>> {
        let (doc_type, input) = DocType::into_parser()
            .map(|v| XmlNode::DocType(v))
            .ok()
            .parse(self.input.clone())?;

        self.input = input;

        Ok(doc_type)
    }

    #[inline(always)]
    fn read_misc(&mut self) -> Result<Option<XmlNode<I>>, ControlFlow<ReadError<I>>> {
        let (misc, input) = PI::into_parser()
            .map(|v| XmlNode::PI(v))
            .or(Comment::into_parser().map(|v| XmlNode::Comment(v)))
            .or(ensure_ws.map(|v| XmlNode::S(v)))
            .ok()
            .parse(self.input.clone())?;

        self.input = input;

        Ok(misc)
    }

    #[inline(always)]
    fn read_root_el(&mut self) -> Result<XmlNode<I>, ControlFlow<ReadError<I>>> {
        let (el, input) = ElemStart::parse(self.input.clone())?;

        self.input = input;

        if el.is_empty {
            self.state = ReadState::MiscAfterElement;
        } else {
            self.starts += 1;
            self.state = ReadState::Element;
        }

        return Ok(XmlNode::Start(el));
    }

    #[inline(always)]
    fn read_el(&mut self) -> Result<XmlNode<I>, ControlFlow<ReadError<I>>> {
        let (node, input) = ElemEnd::into_parser()
            .map(|v| XmlNode::End(v))
            .or(ElemStart::into_parser().map(|v| XmlNode::Start(v)))
            .or(PI::into_parser().map(|v| XmlNode::PI(v)))
            .or(Comment::into_parser().map(|v| XmlNode::Comment(v)))
            .or(CData::into_parser().map(|v| XmlNode::CData(v)))
            .or(CharData::into_parser().map(|v| XmlNode::CharData(v)))
            .parse(self.input.clone())?;

        self.input = input;

        match &node {
            XmlNode::Start(_) => {
                self.starts += 1;
            }
            XmlNode::End(_) => {
                self.starts -= 1;
            }
            _ => {}
        }

        if self.starts == 0 {
            self.state = ReadState::MiscAfterElement;
        }

        return Ok(node);
    }
}

impl<I> XmlReader<I>
where
    I: Input<Item = u8> + AsBytes + Clone + Debug,
{
    /// Create a new reader.
    pub fn new(state: ReadState, input: I) -> Self {
        Self {
            state,
            input,
            starts: 0,
        }
    }

    /// read next xml node.
    #[inline(always)]
    pub fn read_next(&mut self) -> Result<Option<XmlNode<I>>, ControlFlow<ReadError<I>>> {
        loop {
            match self.state {
                ReadState::XmlDecl => return self.read_xml_decl().map(|v| Some(v)),
                ReadState::MiscBeforeDocType => {
                    if let Some(misc) = self.read_misc()? {
                        return Ok(Some(misc));
                    } else {
                        self.state = ReadState::DocType;
                        continue;
                    }
                }
                ReadState::DocType => {
                    if let Some(doctype) = self.read_doctype()? {
                        self.state = ReadState::MiscBeforeElement;
                        return Ok(Some(doctype));
                    } else {
                        self.state = ReadState::RootElement;
                        continue;
                    }
                }
                ReadState::MiscBeforeElement => {
                    if let Some(misc) = self.read_misc()? {
                        return Ok(Some(misc));
                    } else {
                        self.state = ReadState::RootElement;
                        continue;
                    }
                }
                ReadState::RootElement => {
                    return self.read_root_el().map(|v| Some(v));
                }
                ReadState::Element => {
                    return self.read_el().map(|v| Some(v));
                }
                ReadState::MiscAfterElement => {
                    if let Some(misc) = self.read_misc()? {
                        return Ok(Some(misc));
                    } else {
                        self.state = ReadState::Eof;
                        continue;
                    }
                }
                ReadState::Eof => return Ok(None),
            }
        }
    }
}

impl<I> Iterator for XmlReader<I>
where
    I: Input<Item = u8> + AsBytes + Clone + Debug,
{
    type Item = Result<XmlNode<I>, ControlFlow<ReadError<I>>>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next() {
            Ok(v) => v.map(|v| Ok(v)),
            Err(err) => Some(Err(err)),
        }
    }
}
