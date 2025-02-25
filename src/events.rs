//! Events for serializing/deserializing xml document.

use std::borrow::Cow;

use parserc::Span;

/// Xml node name: local_name and optional prefix.
#[derive(Debug, PartialEq, Clone)]
pub struct Name<'a> {
    /// A local name, e.g. `string`` in `xsi:string`.
    pub local_name: Cow<'a, str>,
    /// A name prefix, e.g. `xsi` in `xsi:string`.
    pub prefix: Option<Cow<'a, str>>,
}

impl<'a, L> From<L> for Name<'a>
where
    Cow<'a, str>: From<L>,
{
    fn from(value: L) -> Self {
        Self {
            local_name: value.into(),
            prefix: None,
        }
    }
}

impl<'a> Name<'a> {
    /// Create a new node `Name` with `prefix` and `local_name`.
    pub fn new<P, L>(prefix: P, local_name: L) -> Self
    where
        Cow<'a, str>: From<P> + From<L>,
    {
        Self {
            local_name: local_name.into(),
            prefix: Some(prefix.into()),
        }
    }
    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    ///
    pub fn into_owned(self) -> Name<'static> {
        Name {
            local_name: self.local_name.into_owned().into(),
            prefix: self.prefix.map(|n| n.into_owned().into()),
        }
    }
}

/// Represents the xml version num: 1.0 or 1.1
#[derive(Debug, PartialEq, Clone)]
pub enum XmlVersion {
    Ver10,
    Ver11,
}

/// Events for serializing/deserializing xml document.
#[derive(Debug, PartialEq, Clone)]
pub enum Event<'a> {
    /// XMLDecl: '<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'
    XmlDecl {
        /// VersionInfo: `1.0 or 1.1`. see [`XmlVersion`]
        version: XmlVersion,
        /// EncodingDecl: encoding="utf-8"
        encoding: Option<Cow<'a, str>>,
        /// `yes` or `no`
        standalone: Option<bool>,
        /// Code span of `XmlDecl`.
        span: Option<Span>,
    },
    /// Element node: <a...
    Element(Name<'a>, Option<Span>),
    /// Attr node: a=...
    Attr {
        /// Attribute name.
        name: Name<'a>,
        /// Attr value.
        value: Cow<'a, str>,
        /// Code span of this node.
        span: Option<Span>,
    },
    /// #Text node..
    Text(Cow<'a, str>, Option<Span>),
    /// CData node...
    CData(Cow<'a, str>, Option<Span>),

    /// Processing Instruction: <?xml version="1.0" encoding="UTF-8" ?>
    ProcessingInstruction(Name<'a>, Option<Span>),

    /// Comment node: <!-- xxxxx -->
    Comment(Cow<'a, str>, Option<Span>),

    /// Unparsed document type node: <!DOCTYPE book [<!ENTITY h 'hardcover'>]>
    DocumentType(Cow<'a, str>, Option<Span>),

    /// Unparsed notation node: <notation id = ID...> ... </notation>
    Notation(Cow<'a, str>, Option<Span>),

    /// Pop a node that may has children.
    Pop(Option<Span>),
}

impl<'a> Event<'a> {
    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    ///
    pub fn into_owned(self) -> Event<'static> {
        match self {
            Event::XmlDecl {
                version,
                encoding,
                standalone,
                span,
            } => Event::XmlDecl {
                version,
                encoding: encoding.map(|v| v.into_owned().into()),
                standalone,
                span,
            },
            Event::Element(name, span) => Event::Element(name.into_owned(), span),
            Event::Attr { name, value, span } => Event::Attr {
                name: name.into_owned(),
                value: value.into_owned().into(),
                span,
            },
            Event::Text(cow, span) => Event::Text(cow.into_owned().into(), span),
            Event::CData(cow, span) => Event::CData(cow.into_owned().into(), span),
            Event::ProcessingInstruction(name, span) => {
                Event::ProcessingInstruction(name.into_owned(), span)
            }
            Event::Comment(cow, span) => Event::Comment(cow.into_owned().into(), span),
            Event::DocumentType(cow, span) => Event::DocumentType(cow.into_owned().into(), span),
            Event::Notation(cow, span) => Event::Notation(cow.into_owned().into(), span),
            Event::Pop(span) => Event::Pop(span),
        }
    }
}

impl<'a> Event<'a> {
    /// Create a `xmldecl` event.
    pub fn xml_decl<E>(version: XmlVersion, encoding: E, standalone: bool) -> Self
    where
        Cow<'a, str>: From<E>,
    {
        Self::XmlDecl {
            version,
            encoding: Some(encoding.into()),
            standalone: Some(standalone),
            span: None,
        }
    }

    /// Create a `xmldecl` event.
    pub fn xml_decl_with_span<E>(
        version: XmlVersion,
        encoding: E,
        standalone: bool,
        span: Span,
    ) -> Self
    where
        Cow<'a, str>: From<E>,
    {
        Self::XmlDecl {
            version,
            encoding: Some(encoding.into()),
            standalone: Some(standalone),
            span: Some(span),
        }
    }

    /// Create a `element` event.
    pub fn element<N>(name: N) -> Self
    where
        Name<'a>: From<N>,
    {
        Self::Element(name.into(), None)
    }

    /// Create a `element` event.
    pub fn element_with_span<N>(name: N, span: Span) -> Self
    where
        Name<'a>: From<N>,
    {
        Self::Element(name.into(), Some(span))
    }

    /// Create a `attr` event.
    pub fn attr<N, V>(name: N, value: V) -> Self
    where
        Name<'a>: From<N>,
        Cow<'a, str>: From<V>,
    {
        Self::Attr {
            name: name.into(),
            value: value.into(),
            span: None,
        }
    }

    /// Create a `attr` event.
    pub fn attr_with_span<N, V>(name: N, value: V, span: Span) -> Self
    where
        Name<'a>: From<N>,
        Cow<'a, str>: From<V>,
    {
        Self::Attr {
            name: name.into(),
            value: value.into(),
            span: Some(span),
        }
    }

    /// Create a `text` event.
    pub fn text<V>(value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::Text(value.into(), None)
    }

    /// Create a `text` event.
    pub fn text_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::Text(value.into(), Some(span))
    }

    /// Create a `cdata` event.
    pub fn cdata<V>(value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::CData(value.into(), None)
    }

    /// Create a `cdata` event.
    pub fn cdata_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::CData(value.into(), Some(span))
    }

    /// Create a `processing instruction` event.
    pub fn processing_instruction<V>(value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::ProcessingInstruction(value.into(), None)
    }

    /// Create a `processing instruction` event.
    pub fn processing_instruction_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::ProcessingInstruction(value.into(), Some(span))
    }

    /// Create a `comment` event.
    pub fn comment_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::Comment(value.into(), Some(span))
    }

    /// Create a `document type` event.
    pub fn document_type_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::DocumentType(value.into(), Some(span))
    }

    /// Create a `notation` event.
    pub fn notation<V>(value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::Notation(value.into(), None)
    }

    /// Create a `notation` event.
    pub fn notation_with_span<V>(value: V, span: Span) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        Self::Notation(value.into(), Some(span))
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn test_events() {
        assert_eq!(
            Event::attr("hello", "world"),
            Event::Attr {
                name: "hello".into(),
                value: "world".into(),
                span: None
            }
        );
    }
}
