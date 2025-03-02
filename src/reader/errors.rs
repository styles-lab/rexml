use parserc::Span;

/// Error returns by this module.
#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadError {
    /// Error from [`parserc`]
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),

    /// Types of error that can be raised when parsing `xmldecl`
    #[error("Parse `xmldecl` error: {0} {1}")]
    XmlDecl(ReadKind, Span),

    /// Types of error that can be raised when parsing `element` start tag.
    #[error("Parse `element` start tag error: {0} {1}")]
    StartTag(ReadKind, Span),

    /// Types of error that can be raised when parsing `element` end tag.
    #[error("Parse `element` end tag error: {0} {1}")]
    EndTag(ReadKind, Span),

    /// Types of error that can be raised when parsing `pi`
    #[error("Parse `pi` error: {0} {1}")]
    PI(ReadKind, Span),

    /// Types of error that can be raised when parsing `comment`
    #[error("Parse `comment` error: {0} {1}")]
    Comment(ReadKind, Span),

    /// Types of error that can be raised when parsing `cdata`
    #[error("Parse `cdata` error: {0} {1}")]
    CData(ReadKind, Span),

    /// Types of error that can be raised when parsing `chardata`
    #[error("Parse `chardata` error: {0} {1}")]
    CharData(ReadKind, Span),

    /// Types of error that can be raised when parsing `attr`
    #[error("Parse `attr` error: {0} {1}")]
    Attr(ReadKind, Span),

    /// Types of error that can be raised when parsing `name`
    #[error("Parse `name` error: {0} {1}")]
    Name(ReadKind, Span),

    /// Types of error that can be raised when parsing `whitespace`
    #[error("Expect whitespace.")]
    WS(Span),

    /// Types of error that can be raised when parsing `quote string`
    #[error("Parse `quote` error: {0} {1}")]
    Quote(ReadKind, Span),

    #[error("Parse `version` error: {0} {1}")]
    Version(ReadKind, Span),

    #[error("Parse `encoding` error: {0} {1}")]
    Encoding(ReadKind, Span),

    #[error("Parse `standalone` error: {0} {1}")]
    Standalone(ReadKind, Span),

    #[error("Parse `doctype` error: {0} {1}")]
    DocType(ReadKind, Span),

    #[error("stag({0}) and etag({1}) do not match")]
    ElemTagMismatch(Span, Span),

    #[error("No `stag` found for etag({0})")]
    MissStartTag(Span),
}

impl parserc::ParseError for ReadError {}

/// Read error kind.
#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadKind {
    #[error("target is none.")]
    None,

    #[error("expect `1.1` or `1.0`")]
    LitVersion,

    #[error("expect {0}")]
    LitStr(&'static str),

    /// expect a prefix token
    #[error("expect prefix {0}")]
    Prefix(&'static str),

    /// expect a suffix token
    #[error("expect suffix {0}")]
    Suffix(&'static str),

    /// expect whitespace token.
    #[error("expect suffix `whitespace`")]
    WS,

    /// expect eq token.
    #[error("expect `S? '=' S?`")]
    Eq,

    /// PI target is reserved word .
    #[error("reserved word `(('X' | 'x') ('M' | 'm') ('L' | 'l'))`")]
    Reserved,

    /// namespace prefix of tag/attribute/pi_target/.. name.
    #[error("expect namespace prefix")]
    NamespacePrefix,

    /// expect local_name part of `name`.
    #[error("expect local_name part")]
    LocalName,

    /// expect target name of the `PI`.
    #[error("expect target name")]
    PITarget,

    /// expect body of the `PI`.
    #[error("expect body")]
    PIBody,

    #[error("expect encoding name")]
    EncName,

    #[error("expect tag name.")]
    TagName,

    #[error("quote string is incomplete.")]
    Quote,
}
