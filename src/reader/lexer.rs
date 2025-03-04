//! XML tokenizer.

use std::{borrow::Cow, num::NonZeroUsize};

use memchr::{memchr, memmem};
use parserc::Span;

use super::LexerError;

#[inline]
fn is_ws(c: u8) -> bool {
    matches!(c, b'\x20' | b'\x09' | b'\x0d' | b'\x0a')
}

/// The token type returns by [`XmLexer`].
#[derive(Debug, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum XmlToken {
    /// Unparsed doctype entity.
    /// See [`DocType`](https://www.w3.org/TR/xml11/#NT-doctypedecl)
    DocTypeStart(Span),
    /// non-`S` and non-`markup` chars.
    Chars(Span),
    /// <?
    PIStart(Span),
    /// ?>
    PIEnd(Span),
    /// See [`CData`](https://www.w3.org/TR/xml11/#NT-CData)
    CData(Span),
    /// See [`Comment`](https://www.w3.org/TR/xml11/#NT-Comment)
    Comment(Span),
    /// See [`WhiteSpace`](https://www.w3.org/TR/xml11/#NT-S)
    WS(Span),
    /// See [`Eq`](https://www.w3.org/TR/xml11/#NT-Eq)
    Eq(Span),
    /// `<` Name
    StartTag(Span),
    /// `>`
    EndTag(Span),
    /// `/>`
    EmptyTag(Span),
    /// `</`
    ElementEndStartTag(Span),
    /// 'xxx...' or "xxx..."
    QuoteStr { double_quote: bool, content: Span },
}

/// The lexer for xml document.
#[allow(unused)]
pub struct XmLexer<'a> {
    /// The segment of the input xml document.
    input: Cow<'a, str>,
    /// the segment span in the xml document.
    span: Span,
    // input length.
    len: usize,
    /// read offset.
    offset: usize,
    /// current line no.
    lines: NonZeroUsize,
    /// current col no.
    cols: NonZeroUsize,
    /// true to capture `chardata`.
    capture_char_data: bool,
}

impl<'a> From<&'a str> for XmLexer<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value.into(), Span::new(0, value.len(), 1, 1))
    }
}

impl<'a> From<&'a String> for XmLexer<'a> {
    fn from(value: &'a String) -> Self {
        Self::new(value.into(), Span::new(0, value.len(), 1, 1))
    }
}

impl From<String> for XmLexer<'static> {
    fn from(value: String) -> Self {
        let span = Span::new(0, value.len(), 1, 1);
        Self::new(value.into(), span)
    }
}

impl<'a> XmLexer<'a> {
    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.offset == self.len {
            return None;
        }

        let byte = self.input.as_bytes()[self.offset];
        self.offset += 1;

        if byte == b'\n' {
            self.lines.checked_add(1).unwrap();
            self.cols = NonZeroUsize::new(1).unwrap();
        }

        Some(byte)
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        if self.offset == self.len {
            return None;
        }

        let byte = self.input.as_bytes()[self.offset];

        Some(byte)
    }

    #[allow(unused)]
    fn seek(&mut self, span: Span) {
        self.offset = span.offset - self.span.offset;
        self.lines = NonZeroUsize::new(span.lines).expect("seek: lines > 0");
        self.cols = NonZeroUsize::new(span.lines).expect("seek: cols > 0");
    }

    fn advance(&mut self, steps: usize) {
        self.offset += steps;
        assert!(
            self.offset <= self.len,
            "advance: out of range {}",
            self.offset
        );
    }

    /// Parse `S` .
    fn next_ws(&mut self) -> XmlToken {
        let mut span = self.span();
        span.len = 0;
        while let Some(next) = self.next() {
            if !is_ws(next) {
                break;
            }

            span.len += 1;
        }

        assert!(span.len > 0, "directly call ws fn.");

        XmlToken::WS(span)
    }

    fn next_pi_start(&mut self) -> Option<XmlToken> {
        if self.remaining() < 2 {
            return None;
        }

        if self.unparsing_as_bytes_with(2) == b"<?" {
            let mut span = self.span();

            span.len = 2;

            self.advance(2);

            return Some(XmlToken::PIStart(span));
        }

        None
    }

    fn next_element_end_start_tag(&mut self) -> Option<XmlToken> {
        if self.remaining() < 2 {
            return None;
        }

        if self.unparsing_as_bytes_with(2) == b"</" {
            let mut span = self.span();

            span.len = 2;

            self.advance(2);

            return Some(XmlToken::ElementEndStartTag(span));
        }

        None
    }

    fn next_cdata(&mut self) -> Result<Option<XmlToken>, LexerError> {
        if self.remaining() < 9 {
            return Ok(None);
        }

        if self.unparsing_as_bytes_with(9) == b"<![CDATA[" {
            self.advance(9);
            let mut span = self.span();

            if let Some(offset) = memmem::find(self.unparsing().as_bytes(), b"]]>") {
                span.len = offset;
                for _ in 0..(offset + 3) {
                    self.next();
                }

                return Ok(Some(XmlToken::CData(span)));
            }
        }

        Ok(None)
    }

    fn next_comment(&mut self) -> Result<Option<XmlToken>, LexerError> {
        if self.remaining() < 4 {
            return Ok(None);
        }

        if self.unparsing_as_bytes_with(4) == b"<!--" {
            self.advance(4);
            let mut span = self.span();

            if let Some(offset) = memmem::find(self.unparsing().as_bytes(), b"-->") {
                span.len = offset;
                for _ in 0..(offset + 3) {
                    self.next();
                }

                return Ok(Some(XmlToken::Comment(span)));
            }
        }

        Ok(None)
    }

    fn next_doc_type_start(&mut self) -> Option<XmlToken> {
        if self.remaining() < 9 {
            return None;
        }

        if self.unparsing_as_bytes_with(9) == b"<!DOCTYPE" {
            let mut span = self.span();
            span.len = 9;
            self.advance(9);

            return Some(XmlToken::DocTypeStart(span));
        }

        None
    }

    fn next_pi_end(&mut self) -> Result<XmlToken, LexerError> {
        if self.remaining() < 2 {
            return Err(LexerError::PIEnd(self.span()));
        }

        if self.unparsing_as_bytes_with(2) == b"?>" {
            let mut span = self.span();
            span.len = 2;
            self.advance(2);

            return Ok(XmlToken::PIEnd(span));
        }

        return Err(LexerError::PIEnd(self.span()));
    }

    fn next_empty_tag(&mut self) -> Result<XmlToken, LexerError> {
        if self.remaining() < 2 {
            return Err(LexerError::EmptyTag(self.span()));
        }

        if self.unparsing_as_bytes_with(2) == b"/>" {
            let mut span = self.span();
            span.len = 2;
            self.advance(2);

            return Ok(XmlToken::EmptyTag(span));
        }

        return Err(LexerError::EmptyTag(self.span()));
    }

    fn next_quote_str(&mut self) -> Result<XmlToken, LexerError> {
        let start = self.span();
        let quote = self.next().unwrap();

        if let Some(offset) = memchr(quote, self.unparsing_as_bytes()) {
            let mut span = self.span();

            span.len = offset;

            for _ in 0..(offset + 1) {
                self.next();
            }

            return Ok(XmlToken::QuoteStr {
                double_quote: quote == b'"',
                content: span,
            });
        }

        return Err(LexerError::QuoteStr(start, quote as char));
    }
}

impl<'a> XmLexer<'a> {
    /// Create a new XML lexer.
    pub fn new(input: Cow<'a, str>, span: Span) -> Self {
        assert_eq!(input.len(), span.len());

        Self {
            len: input.len(),
            input,
            span,
            offset: 0,
            lines: NonZeroUsize::new(span.lines).expect("new: lines > 0"),
            cols: NonZeroUsize::new(span.cols).expect("new: cols > 0"),
            capture_char_data: false,
        }
    }

    /// Reset `capture_char_data` flag.
    #[inline]
    pub fn capture_char_data(&mut self, flag: bool) {
        self.capture_char_data = flag;
    }

    /// Returns unparsing input source length.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.len - self.offset
    }

    /// Returns the unparsing part of the input source.
    #[inline]
    pub fn unparsing(&self) -> &str {
        &self.input[self.offset..]
    }

    /// Returns the unparsing part of the input source as bytes.
    #[inline]
    pub fn unparsing_as_bytes_with(&self, len: usize) -> &[u8] {
        self.input[self.offset..self.offset + len].as_bytes()
    }

    /// Returns the unparsing part of the input source as bytes.
    #[inline]
    pub fn unparsing_as_bytes(&self) -> &[u8] {
        self.unparsing().as_bytes()
    }

    /// Returns the next byte's span.
    #[inline]
    fn span(&self) -> Span {
        Span {
            offset: self.span.offset + self.offset,
            len: if self.remaining() == 0 { 0 } else { 1 },
            lines: self.lines.into(),
            cols: self.cols.into(),
        }
    }

    /// Read next token.
    ///
    /// Returns `None`, if the end of the input is reached.
    /// Returns [`Incomplete`](LexerError::Incomplete) error, if could not determine the last token.
    pub fn next_token(&mut self) -> Result<Option<XmlToken>, LexerError> {
        match self.peek() {
            Some(b'<') => {
                if let Some(token) = self.next_element_end_start_tag() {
                    return Ok(Some(token));
                }

                if let Some(token) = self.next_pi_start() {
                    return Ok(Some(token));
                }

                if let Some(token) = self.next_comment()? {
                    return Ok(Some(token));
                }

                if let Some(token) = self.next_cdata()? {
                    return Ok(Some(token));
                }

                if let Some(token) = self.next_doc_type_start() {
                    return Ok(Some(token));
                }

                let span = self.span();
                self.next();

                return Ok(Some(XmlToken::StartTag(span)));
            }
            Some(b'?') => return self.next_pi_end().map(|v| Some(v)),
            Some(b'/') => return self.next_empty_tag().map(|v| Some(v)),
            Some(b'"') | Some(b'\'') => {
                if !self.capture_char_data {
                    return self.next_quote_str().map(|v| Some(v));
                }
            }
            Some(c) => {
                if !self.capture_char_data {
                    if is_ws(c) {
                        return Ok(Some(self.next_ws()));
                    }
                }
            }
            None => return Ok(None),
        }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use parserc::Span;

    use crate::reader::lexer::XmlToken;

    use super::XmLexer;

    #[test]
    fn test_ws() {
        assert_eq!(
            XmLexer::from("  ").next_token(),
            Ok(Some(XmlToken::WS(Span::new(0, 2, 1, 1))))
        );

        assert_eq!(
            XmLexer::from("   \n\t").next_token(),
            Ok(Some(XmlToken::WS(Span::new(0, 5, 1, 1))))
        );
    }
}
