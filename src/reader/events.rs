use parserc::Span;

use crate::types::XmlVersion;

/// See [`Name`](https://www.w3.org/TR/xml11/#NT-Name)
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct Name {
    /// The namespace prefix.
    pub prefix: Option<Span>,
    /// Local part of quified name.
    pub local_name: Span,
}

// /// CharRef ::= '&#' [0-9]+ ';'| '&#x' [0-9a-fA-F]+ ';'
// #[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
// pub enum CharRef {
//     /// '&#' [0-9]+ ';'
//     Digit(Span),
//     /// '&#x' [0-9a-fA-F]+ ';'
//     HexDigit(Span),
// }

// /// EntityRef ::= '&' Name ';'
// ///
// /// See [`Character and Entity References`](https://www.w3.org/TR/xml11/#NT-Reference)
// #[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
// pub struct EntityRef(
//     /// The span of entity name in the source code.
//     pub Name,
// );

// /// See [`Character and Entity References`](https://www.w3.org/TR/xml11/#NT-Reference)
// #[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
// pub enum Ref {
//     CharRef(CharRef),
//     EntityRef(EntityRef),
// }

/// Unparsed chardata my includes `CharRef` or `EntityRef`
///
/// See xml [`CharData`](https://www.w3.org/TR/xml11/#NT-CharData)
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct CharData(
    /// The span of `CharData` in the source code.
    pub Span,
);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct CData(
    /// The span of `CharData` in the source code.
    pub Span,
);

/// See xml [`Comment`](https://www.w3.org/TR/xml11/#NT-Comment)
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct Comment(pub Span);

/// See xml [`PI`](https://www.w3.org/TR/xml11/#NT-PI)
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct PI {
    /// The span of PI target name in the source code.
    pub target: Name,
    /// The span of unparsed content of the PI.
    pub unparsed: Option<Span>,
}
/// White space.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct WS(
    /// The span of white space chars in the source code.
    pub Span,
);

/// See [`Attribute`](https://www.w3.org/TR/xml11/#NT-Attribute)
#[derive(Debug, PartialEq, Clone)]
pub struct Attr {
    /// Attribute name part.
    pub name: Name,
    /// Unparsed attribute value part.
    pub value: Span,
}

/// See [`XmlDecl`](https://www.w3.org/TR/xml11/#NT-XMLDecl)
#[derive(Debug, PartialEq, Clone)]
pub struct XmlDecl {
    /// See xml [`version`](https://www.w3.org/TR/xml11/#NT-VersionInfo)
    pub version: XmlVersion,
    /// See xml [`encoding`](https://www.w3.org/TR/xml11/#NT-EncodingDecl)
    pub encoding: Option<Span>,
    /// See xml [`standalone`](https://www.w3.org/TR/xml11/#NT-SDDecl)
    pub standalone: Option<bool>,
}

/// Unparsed doctype element.
///
/// See [`DocType`](https://www.w3.org/TR/xml11/#NT-doctypedecl)
#[derive(Debug, PartialEq, Clone)]
pub struct DocType(
    /// The span of `CharData` in the source code.
    pub Span,
);

/// Read event returns by xml parsers.
#[derive(Debug, PartialEq, Clone)]
pub enum ReadEvent {
    XmlDecl(XmlDecl),
    DocType(DocType),
    // Ref(Ref),
    PI(PI),
    WS(WS),
    CharData(CharData),
    CData(CData),
    Comment(Comment),
    EmptyElement { name: Name, attrs: Vec<Attr> },
    ElementStart { name: Name, attrs: Vec<Attr> },
    ElementEnd(Name),
}
