use parserc::Span;

/// Error type returns by this module.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReadError {
    /// Lexer error.
    #[error(transparent)]
    Lexer(#[from] LexerError),
}

/// Error type returns by lexer.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LexerError {
    /// Lexer reached the end of the input source, but could not determine the last token.
    #[error("incomplete {0}")]
    Incomplete(Span),

    #[error("No matching `]]>` found for cdata start {0}")]
    CDataEnd(Span),

    #[error("expect `?>` {0}")]
    PIEnd(Span),

    #[error("expect `/>` {0}")]
    EmptyTag(Span),

    #[error("no matching `{1}` found for quote str {0}")]
    QuoteStr(Span, char),
}
