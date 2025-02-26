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
    #[error("`yes` or `no`")]
    SDBool,
    #[error("`encoding name`")]
    EncName,
    #[error("`<!--`")]
    CommentStart,
    #[error("`-->`")]
    CommentEnd,
    #[error("<?")]
    PIStart,
    #[error("`PITarget`")]
    PITarget,
    #[error("`PIUnparsed`")]
    PIUnparsed,
    #[error("`?>`")]
    PIEnd,
    #[error("`('X' | 'x') ('M' | 'm') ('L' | 'l')`")]
    ReservedXml,
    #[error("`NameStartChar`")]
    NameStartChar,
    #[error("`NameChar`")]
    NameChar,
    #[error("start tag `'` or `\"`")]
    QuoteStart,
    #[error("end tag `'` or `\"`")]
    QuoteEnd,
    #[error("`SYSTEM` or `PUBLIC`")]
    ExternalType,
    #[error("`white space`")]
    Ws,
    #[error("`SystemLiteral`")]
    SystemLiteral,
    #[error("`PubIdLiteral`")]
    PubIdLiteral,
}
