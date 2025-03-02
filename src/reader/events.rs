use parserc::Span;

use crate::types::XmlVersion;

/// Unparsed tag/attr/pi/.. name.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Name(
    /// the code span of the name entity.
    pub Span,
);

/// A parsed `XmlDecl` part.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct XmlDecl {
    /// Required `version` part.
    pub version: XmlVersion,
    /// An optional `encoding` part.
    pub encoding: Option<Span>,
    /// An optional `standalone` part.
    pub standalone: Option<bool>,
}

/// An unparsed doctype entity.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct DocType(pub Span);

/// A parsed `EmptyElemTag` or `STag` with unparsed attr list.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Start {
    /// The name of the stag.
    pub name: Name,
    /// Unparsed attribute list.
    pub attrs: Span,
}

/// A parsed attr name/value pair.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Attr {
    pub name: Name,
    pub value: Span,
}

/// An parsed `ETag`
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct End(
    /// The name of the etag.
    pub Name,
);

/// An parsed `comment`
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Comment(pub Span);

/// An parsed `cdata`
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct CData(pub Span);

/// An parsed `whitespace`
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct WS(pub Span);

/// An parsed `cdata` with unparsed reference entities.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct CharData(pub Span);

/// An parsed `PI` section with unparsed body.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct PI {
    /// The target name.
    pub target: Name,
    /// Optional unparsed `PI` body.
    pub unparsed: Option<Span>,
}

/// item of reading iterator.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum ReadEvent {
    XmlDecl(XmlDecl),
    DocType(DocType),
    Comment(Comment),
    CData(CData),
    CharData(CharData),
    PI(PI),
    ElemStart(Start),
    EmptyElem(Start),
    ElemEnd(End),
    WS(WS),
}
