use std::fmt::Debug;

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadError<I> {
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),
    #[error("expect {0} {1}")]
    Expect(ReadKind, I),

    #[error("unexpect {0} {1}")]
    Unexpect(ReadKind, I),
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadKind {
    #[error("`Name`")]
    Name,
    #[error("`=`")]
    Eq,
    #[error("`whitespace`")]
    S,
    #[error("keyword `{0}`")]
    Keyword(&'static str),
    #[error("`version`")]
    Version,
    #[error("`yes` or `no`")]
    YesNo,
    #[error("`encoding`")]
    Encoding,
}
