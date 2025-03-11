use std::fmt::Debug;

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadError<I>
where
    I: Clone,
{
    #[error(transparent)]
    Parserc(#[from] parserc::Kind),
    #[error("expect {0} {1}")]
    Expect(ReadKind, I),
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ReadKind {
    #[error("`=`")]
    Eq,
}
