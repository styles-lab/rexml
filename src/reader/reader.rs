use super::lexer::{LexerError, XmlSpan};

/// Error type returns by this module.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReadError {
    #[error("read {0} error: incomplete {1}")]
    Incomplete(ReadKind, XmlSpan),

    #[error("read {0} error: {1}")]
    Tokenizer(ReadKind, LexerError),

    #[error("read {0} error: expect {1}")]
    Expect(ReadKind, ReadKind),
}

/// Kind of read entity.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReadKind {
    #[error("`attr`")]
    Attr,
    #[error("`'...' or \"..\"` {0}")]
    QuoteStr(XmlSpan),
    #[error("`name` {0}")]
    Name(XmlSpan),
    #[error("`=` {0}")]
    Eq(XmlSpan),
}
