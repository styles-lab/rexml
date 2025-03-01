use parserc::{ParseError, Span};

/// Error type returns by [`read_xml`]
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ReadError {
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),
    #[error("read `VersionInfo` error, expect {0} {1}")]
    Version(ReadKind, Span),
    #[error("read `standalone` error, expect {0} {1}")]
    Standalone(ReadKind, Span),
    #[error("read `encoding` error, expect {0} {1}")]
    Encoding(ReadKind, Span),
    #[error("read `ws` error {0}")]
    Ws(Span),
    #[error("read `eq` error {0}")]
    Eq(Span),
    #[error("read `comment` error, expect {0} {1}")]
    Comment(ReadKind, Span),
    #[error("read `pi` error, expect {0} {1}")]
    PI(ReadKind, Span),
    #[error("read reserved word {0} {1}")]
    ReservedWord(ReadKind, Span),
    #[error("read `name` error, expect {0} {1}")]
    Name(ReadKind, Span),
    #[error("read `quote literal string` error, expect {0} {1}")]
    Quote(ReadKind, Span),

    #[error("read `external-id` error, expect {0} {1}")]
    ExternalId(ReadKind, Span),

    #[error("read `PEReference` error, expect {0} {1}")]
    PERef(ReadKind, Span),
    #[error("read `EntityRef` error, expect {0} {1}")]
    EntityRef(ReadKind, Span),
    #[error("read `CharRef` error, expect {0} {1}")]
    CharRef(ReadKind, Span),
    #[error("read `DocType` error, expect {0} {1}")]
    DocType(ReadKind, Span),

    #[error("read `CData` error, expect {0} {1}")]
    CData(ReadKind, Span),
    #[error("read `Attr` error, expect {0} {1}")]
    Attr(ReadKind, Span),
    #[error("read `CharData` error")]
    CharData,
    #[error("read `XmlDecl` error, expect {0} {1}")]
    XmlDecl(ReadKind, Span),
    #[error("read `Element` error, expect {0} {1}")]
    Element(ReadKind, Span),
}

impl ParseError for ReadError {}

/// Read kind type.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ReadKind {
    #[error("`'` or `\"`")]
    Quote,
    #[error("`1.1` or `1.0`")]
    VerStr,
    #[error("`version=`")]
    LitVer,
    #[error("`LitNum`")]
    LitNum,
    #[error("`yes` or `no`")]
    SDBool,
    #[error("`encoding name`")]
    EncName,
    #[error("`PITarget`")]
    PITarget,
    #[error("`PIUnparsed`")]
    PIUnparsed,
    #[error("`('X' | 'x') ('M' | 'm') ('L' | 'l')`")]
    ReservedXml,
    #[error("`NameStartChar`")]
    NameStartChar,
    #[error("`NameChar`")]
    NameChar,
    #[error("`Name`")]
    Name,
    #[error("`local_name`")]
    LocalName,
    #[error("`SYSTEM` or `PUBLIC`")]
    ExternalType,
    #[error("`white space`")]
    WS,
    #[error("`SystemLiteral`")]
    SystemLiteral,
    #[error("`PubIdLiteral`")]
    PubIdLiteral,
    #[error("`prefix({0})`")]
    Prefix(&'static str),
    #[error("`suffix({0})`")]
    Suffix(&'static str),
    #[error("`split({0})`")]
    Split(&'static str),

    #[error("`=`")]
    Eq,
}
