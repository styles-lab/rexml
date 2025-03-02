pub use parserc::Span;
use std::{borrow::Cow, num::NonZeroUsize};

/// Error type returns by [`XmlReader`].
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd, Clone)]
pub enum ReadError {
    #[error("incomplete xml document, end at {0}")]
    Incomplete(Span),
}

/// Result type returns by [`XmlReader`]
pub type Result<T> = std::result::Result<T, ReadError>;

/// Token type returns by [`next_token`](XmlReader::next_token).
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Token {}

/// Read state used by [`XmlReader`]
#[allow(unused)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum ReadState {
    XmlDecl,
    MiscsAfterXmlDecl,
    DocType,
    MiscsAfterDocType,
    ElementRoot,
    Element,
    MiscsAfterElement,
    Eof,
}

/// A fast xml entities decoder/iterator implementation.
#[allow(unused)]
pub struct XmlReader<'a> {
    input: Cow<'a, str>,
    /// The input buf max length.
    len: usize,
    /// current reading offset. start with '0'
    offset: usize,
    /// tracking the line no. start with '1'
    lines: NonZeroUsize,
    /// tracking the col no. start with `1`
    cols: NonZeroUsize,
    /// The inner reading state.
    state: ReadState,
}

impl<'a> From<&'a str> for XmlReader<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            input: value.into(),
            len: value.len(),
            offset: 0,
            lines: NonZeroUsize::new(1).unwrap(),
            cols: NonZeroUsize::new(1).unwrap(),
            state: ReadState::XmlDecl,
        }
    }
}

impl<'a> From<&'a String> for XmlReader<'a> {
    fn from(value: &'a String) -> Self {
        Self {
            input: value.into(),
            len: value.len(),
            offset: 0,
            lines: NonZeroUsize::new(1).unwrap(),
            cols: NonZeroUsize::new(1).unwrap(),
            state: ReadState::XmlDecl,
        }
    }
}

impl From<String> for XmlReader<'static> {
    fn from(value: String) -> Self {
        Self {
            len: value.len(),
            input: value.into(),
            offset: 0,
            lines: NonZeroUsize::new(1).unwrap(),
            cols: NonZeroUsize::new(1).unwrap(),
            state: ReadState::XmlDecl,
        }
    }
}
impl<'a> XmlReader<'a> {
    fn read_xml_decl(&mut self) -> Result<Token> {
        todo!()
    }

    fn read_miscs(&mut self, _: ReadState) -> Result<Option<Token>> {
        todo!()
    }

    fn read_doc_type(&mut self) -> Result<Token> {
        todo!()
    }

    fn read_root_element(&mut self) -> Result<Token> {
        todo!()
    }

    fn read_element(&mut self) -> Result<Option<Token>> {
        todo!()
    }
}

impl<'a> XmlReader<'a> {
    /// Reset offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Reset lines.
    pub fn with_lines(mut self, lines: NonZeroUsize) -> Self {
        self.lines = lines;
        self
    }

    /// Reset cols.
    pub fn with_cols(mut self, cols: NonZeroUsize) -> Self {
        self.cols = cols;
        self
    }

    /// Returns next token.
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        loop {
            match self.state {
                ReadState::XmlDecl => return self.read_xml_decl().map(|v| Some(v)),
                ReadState::MiscsAfterXmlDecl => {
                    if let Some(v) = self.read_miscs(ReadState::DocType)? {
                        return Ok(Some(v));
                    }

                    continue;
                }
                ReadState::DocType => return self.read_doc_type().map(|v| Some(v)),
                ReadState::MiscsAfterDocType => {
                    if let Some(v) = self.read_miscs(ReadState::ElementRoot)? {
                        return Ok(Some(v));
                    }

                    continue;
                }
                ReadState::ElementRoot => return self.read_root_element().map(|v| Some(v)),
                ReadState::Element => {
                    if let Some(v) = self.read_element()? {
                        return Ok(Some(v));
                    }

                    continue;
                }
                ReadState::MiscsAfterElement => {
                    if let Some(v) = self.read_miscs(ReadState::Eof)? {
                        return Ok(Some(v));
                    }

                    continue;
                }
                ReadState::Eof => return Ok(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        _ = XmlReader::from("hello world".to_string());
    }
}
