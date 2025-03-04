use std::borrow::Cow;

use parserc::Span;

/// Lexer may raise this error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LexerError {
    #[error("invalid `name` token. {0}")]
    Name(Span),
    #[error("no matching found of `]]>` end tag. {0}")]
    CData(Span),
    #[error("no matching found of `-->` end tag. {0}")]
    Comment(Span),
    #[error("unclosed doc_type declaration. {0}")]
    DocType(Span),
    #[error("expect `?>`. {0}")]
    PIEnd(Span),

    #[error("expect `/>`. {0}")]
    EmptyTag(Span),
    #[error("no matching found of `{0}` end tag. {1}")]
    QuoteStr(char, Span),
}

/// The variant of xml token.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum XmlToken {
    /// `<`
    ElementOpenStartTag(Span),
    /// `</`
    ElementCloseStartTag(Span),
    /// `>`
    EndTag(Span),
    /// `/>`
    EmptyTag(Span),
    /// `<?`
    PIStart(Span),
    /// `?>`
    PIEnd(Span),
    /// `<![CDATA[` Cdata `]]>`
    CData(Span),
    /// `<!--` Cdata `-->`
    Comment(Span),
    /// (#x20 | #x9 | #xD | #xA)+
    WS(Span),
    /// See [`CharData`](https://www.w3.org/TR/xml11/#NT-CharData)
    CharData(Span),
    /// See [`Name`](https://www.w3.org/TR/xml11/#NT-Name)
    Name(Span),
    /// '<!DOCTYPE' ... '>'
    /// See [`doctypedecl`](https://www.w3.org/TR/xml11/#NT-doctypedecl)
    DocType(Span),
    /// `"` ... `"` | `'` ... `'`
    QuoteStr(Span),
    /// `=`
    Eq(Span),
}

/// Read state.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum XmLexerState {
    Markup,
    CharData,
}

/// The lexer for xml document.
///
/// The lexer has two status:
/// * chardata mod: parse all non-markup chars as chardata.
/// * markup mod: parse all non-markup and non-whitespace chars as `Name` or `QuoteStr`
pub struct XmLexer<'a> {
    /// current read state of this lexer.
    state: XmLexerState,
    /// input xml document fragement.
    input: Cow<'a, str>,
    /// The span of the input xml document fragement.
    span: Span,
    /// The cursor offset in the fragement.
    offset: usize,
    /// Current line no. of the read cursor.
    lines: usize,
    /// Current column no. of the read cursor.
    cols: usize,
}

impl<'a> From<&'a str> for XmLexer<'a> {
    fn from(value: &'a str) -> Self {
        let span = Span::new(0, value.len(), 1, 1);
        Self::new(value, span)
    }
}

impl<'a> From<&'a String> for XmLexer<'a> {
    fn from(value: &'a String) -> Self {
        let span = Span::new(0, value.len(), 1, 1);
        Self::new(value, span)
    }
}

impl From<String> for XmLexer<'static> {
    fn from(value: String) -> Self {
        let span = Span::new(0, value.len(), 1, 1);
        Self::new(value, span)
    }
}

#[allow(unused)]
impl<'a> XmLexer<'a> {
    /// Returns next byte in the source code and move the cursor forward one step.
    fn next(&mut self) -> Option<u8> {
        if let Some(c) = self.peek() {
            self.offset += 1;

            if c == b'\n' {
                self.lines += 1;
                self.cols = 1;
            } else {
                self.cols += 1;
            }

            Some(c)
        } else {
            None
        }
    }

    /// Peek next bytes in the source code, but does not move the read cursor.
    fn peek(&mut self) -> Option<u8> {
        if self.offset == self.span.len {
            return None;
        }

        let value = self.unparsed_bytes()[0];

        Some(value)
    }

    /// Read next ws chars.
    fn next_ws(&mut self) -> Option<XmlToken> {
        assert_eq!(self.state, XmLexerState::Markup);

        let mut start = self.next_span();
        start.len = 0;

        while let Some(c) = self.peek() {
            if is_ws(c) {
                self.next();
            } else {
                break;
            }
        }

        let span = start.extend_to(self.next_span());

        if span.len() > 0 {
            Some(XmlToken::WS(span))
        } else {
            None
        }
    }

    /// Read next ws chars.
    fn next_name(&mut self) -> Result<XmlToken, LexerError> {
        assert_eq!(self.state, XmLexerState::Markup);

        let mut start = self.next_span();
        start.len = 0;

        let c = self.next().expect("Must(next_name): length > 0");

        if is_ws(c) || is_markup_char(c) || c == b'-' || c == b'.' || c == b'=' {
            return Err(LexerError::Name(start));
        }

        while let Some(c) = self.peek() {
            if is_ws(c) || is_markup_char(c) || c == b'=' {
                break;
            }

            self.next();
        }

        Ok(XmlToken::Name(start.extend_to(self.next_span())))
    }

    fn next_chardata(&mut self) -> Option<XmlToken> {
        assert_eq!(self.state, XmLexerState::CharData);

        let mut start = self.next_span();
        start.len = 0;

        while let Some(c) = self.peek() {
            if c == b'<' {
                break;
            }

            self.next();
        }

        let span = start.extend_to(self.next_span());

        self.state = XmLexerState::Markup;

        if span.len() > 0 {
            Some(XmlToken::CharData(span))
        } else {
            None
        }
    }

    fn next_cdata(&mut self) -> Result<Option<XmlToken>, LexerError> {
        assert_eq!(self.state, XmLexerState::Markup);

        if self.remaining() < 9 {
            return Ok(None);
        }

        if self.unparsed_bytes_with(9) != b"<![CDATA[" {
            return Ok(None);
        }

        let mut start = self.next_span();
        // Safety: already check by `if self.unparsed_bytes_with..`
        start.offset += 9;
        start.len = 0;

        self.seek(start);

        while let Some(c) = self.next() {
            if c == b']' {
                let mut markers = self.next_span();
                // Safety..
                markers.offset -= 1;

                while let Some(c) = self.next() {
                    if c != b']' {
                        let mut end = self.next_span();

                        markers = markers.extend_to(end);

                        if markers.len() > 2 && c == b'>' {
                            markers.len -= 3;
                            self.state = XmLexerState::CharData;
                            return Ok(Some(XmlToken::CData(start.extend_to_inclusive(markers))));
                        }

                        break;
                    }
                }
            }
        }

        Err(LexerError::CData(start))
    }

    fn next_comment(&mut self) -> Result<Option<XmlToken>, LexerError> {
        assert_eq!(self.state, XmLexerState::Markup);

        if self.remaining() < 4 {
            return Ok(None);
        }

        if self.unparsed_bytes_with(4) != b"<!--" {
            return Ok(None);
        }

        let mut start = self.next_span();
        // Safety: already check by `if self.unparsed_bytes_with..`
        start.offset += 4;
        start.len = 0;

        self.seek(start);

        while let Some(c) = self.next() {
            if c == b'-' {
                let mut markers = self.next_span();
                // Safety..
                markers.offset -= 1;

                while let Some(c) = self.next() {
                    if c != b'-' {
                        let mut end = self.next_span();

                        markers = markers.extend_to(end);

                        if markers.len() > 2 && c == b'>' {
                            markers.len -= 3;
                            self.state = XmLexerState::CharData;
                            return Ok(Some(XmlToken::Comment(start.extend_to_inclusive(markers))));
                        }

                        break;
                    }
                }
            }
        }

        Err(LexerError::Comment(start))
    }

    fn next_quote_str(&mut self) -> Result<XmlToken, LexerError> {
        assert_eq!(self.state, XmLexerState::Markup);

        let quote = self.next().unwrap();

        assert!(quote == b'\'' || quote == b'"');

        let mut start = self.next_span();
        start.len = 0;

        while let Some(c) = self.next() {
            if c == quote {
                let mut end = self.next_span();
                end.offset -= 1;

                return Ok(XmlToken::QuoteStr(start.extend_to(end)));
            }
        }

        Err(LexerError::QuoteStr(
            quote as char,
            start.extend_to(self.next_span()),
        ))
    }

    fn next_doc_type(&mut self) -> Option<XmlToken> {
        assert_eq!(self.state, XmLexerState::Markup);

        if self.remaining() < 9 {
            return None;
        }

        if self.unparsed_bytes_with(9) != b"<!DOCTYPE" {
            return None;
        }

        let mut start = self.next_span();
        start.offset += 9;
        start.len = 0;

        self.seek(start);

        let mut unclosed = 1;

        while let Some(c) = self.peek() {
            if c == b'"' || c == b'\'' {
                self.next_quote_str();
                continue;
            }

            self.next();

            if c == b'<' {
                unclosed += 1;
            }

            if c == b'>' {
                unclosed -= 1;
                if unclosed == 0 {
                    let mut end = self.next_span();
                    end.offset -= 1;
                    self.state = XmLexerState::CharData;
                    return Some(XmlToken::DocType(start.extend_to(end)));
                }
            }
        }

        None
    }

    fn next_pi_start(&mut self) -> Option<XmlToken> {
        assert_eq!(self.state, XmLexerState::Markup);
        if self.remaining() < 2 {
            return None;
        }

        if self.unparsed_bytes_with(2) != b"<?" {
            return None;
        }

        let mut span = self.next_span();
        span.len = 2;

        let mut seek = self.next_span();
        seek.offset += 2;

        self.seek(seek);

        Some(XmlToken::PIStart(span))
    }

    fn next_pi_end(&mut self) -> Result<XmlToken, LexerError> {
        if self.remaining() < 2 {
            return Err(LexerError::PIEnd(self.next_span()));
        }

        if self.unparsed_bytes_with(2) != b"?>" {
            return Err(LexerError::PIEnd(self.next_span()));
        }

        let mut span = self.next_span();
        span.len = 2;

        let mut seek = self.next_span();
        seek.offset += 2;

        self.seek(seek);

        Ok(XmlToken::PIEnd(span))
    }

    fn next_element_close_start_tag(&mut self) -> Option<XmlToken> {
        if self.remaining() < 2 {
            return None;
        }

        if self.unparsed_bytes_with(2) != b"</" {
            return None;
        }

        let mut span = self.next_span();
        span.len = 2;

        let mut seek = self.next_span();
        seek.offset += 2;

        self.seek(seek);

        Some(XmlToken::ElementCloseStartTag(span))
    }

    fn next_empty_tag(&mut self) -> Result<XmlToken, LexerError> {
        if self.remaining() < 2 {
            return Err(LexerError::EmptyTag(self.next_span()));
        }

        if self.unparsed_bytes_with(2) != b"/>" {
            return Err(LexerError::EmptyTag(self.next_span()));
        }

        let mut span = self.next_span();
        span.len = 2;

        let mut seek = self.next_span();
        seek.offset += 2;

        self.seek(seek);

        Ok(XmlToken::EmptyTag(span))
    }
}

impl<'a> XmLexer<'a> {
    /// Create a new `XmLexer` with code `Span`.
    pub fn new<C>(input: C, span: Span) -> Self
    where
        Cow<'a, str>: From<C>,
    {
        let input: Cow<'a, str> = input.into();

        assert_eq!(input.len(), span.len(), "Must: input::len == span::len");
        Self {
            state: XmLexerState::Markup,
            input,
            span,
            offset: 0,
            lines: span.lines,
            cols: span.cols,
        }
    }

    /// Reset the `lexer` mod to `CharData`
    pub fn with_chardata(mut self) -> Self {
        self.state = XmLexerState::CharData;
        self
    }

    /// Reset the `lexer` mod to `Markup`
    pub fn with_markup(mut self) -> Self {
        self.state = XmLexerState::Markup;
        self
    }

    /// Reset the `lexer` mod to `CharData`
    pub fn chardata(&mut self) {
        self.state = XmLexerState::CharData;
    }

    /// Reset the `lexer` mod to `Markup`
    pub fn markup(&mut self) {
        self.state = XmLexerState::Markup;
    }

    /// Returns unparsing source code length.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.span.len - self.offset
    }

    /// Returns unparsed source code as str slice.
    #[inline]
    pub fn unparsed(&self) -> &str {
        &self.input[self.offset..]
    }

    /// Returns unparsed source code as bytes slice.
    #[inline]
    pub fn unparsed_bytes(&self) -> &[u8] {
        &self.input.as_bytes()[self.offset..]
    }

    /// Returns unparsed source code as str slice, up to `len`.
    #[inline]
    pub fn unparsed_with(&self, len: usize) -> &str {
        let mut end = self.offset + len;

        if end > self.span.len {
            end = self.span.len;
        }

        &self.input[self.offset..end]
    }

    /// Returns unparsed source code as bytes slice, up to `len`.
    #[inline]
    pub fn unparsed_bytes_with(&self, len: usize) -> &[u8] {
        let mut end = self.offset + len;

        if end > self.span.len {
            end = self.span.len;
        }

        &self.input.as_bytes()[self.offset..end]
    }

    /// Returns the span of next byte in the source code.
    ///
    /// The `len` of the returned `Span` is zero, if `eof` is reached.
    pub fn next_span(&self) -> Span {
        Span {
            offset: self.span.offset + self.offset,
            len: if self.remaining() > 0 { 1 } else { 0 },
            lines: self.lines,
            cols: self.cols,
        }
    }

    /// Move the read cursor to the span's start position.
    pub fn seek(&mut self, span: Span) {
        self.offset = span.offset - self.span.offset;
        self.lines = span.lines;
        self.cols = span.cols;
    }

    /// Iterate over the source code and returns next token.
    pub fn next_token(&mut self) -> Result<Option<XmlToken>, LexerError> {
        loop {
            match self.state {
                XmLexerState::Markup => match self.peek() {
                    Some(b'<') => {
                        if let Some(token) = self.next_cdata()? {
                            return Ok(Some(token));
                        }

                        if let Some(token) = self.next_comment()? {
                            return Ok(Some(token));
                        }

                        if let Some(token) = self.next_pi_start() {
                            return Ok(Some(token));
                        }

                        if let Some(token) = self.next_element_close_start_tag() {
                            return Ok(Some(token));
                        }

                        let span = self.next_span();
                        self.next();

                        return Ok(Some(XmlToken::ElementOpenStartTag(span)));
                    }
                    Some(b'?') => return self.next_pi_end().map(|v| Some(v)),
                    Some(b'/') => return self.next_empty_tag().map(|v| Some(v)),
                    Some(b'=') => {
                        let span = self.next_span();
                        self.next();
                        return Ok(Some(XmlToken::Eq(span)));
                    }
                    Some(b'"') | Some(b'\'') => return self.next_quote_str().map(|v| Some(v)),
                    Some(b'>') => {
                        let span = self.next_span();
                        self.next();
                        return Ok(Some(XmlToken::EndTag(span)));
                    }
                    Some(_) => {
                        if let Some(token) = self.next_ws() {
                            return Ok(Some(token));
                        }

                        return self.next_name().map(|v| Some(v));
                    }
                    None => {
                        // The eof reached.
                        return Ok(None);
                    }
                },
                XmLexerState::CharData => {
                    if let Some(chardata) = self.next_chardata() {
                        return Ok(Some(chardata));
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for XmLexer<'a> {
    type Item = Result<XmlToken, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(v) => v.map(|v| Ok(v)),
            Err(err) => Some(Err(err)),
        }
    }
}

#[inline]
fn is_ws(c: u8) -> bool {
    matches!(c, b'\x20' | b'\x09' | b'\x0d' | b'\x0a')
}

#[inline]
fn is_markup_char(c: u8) -> bool {
    matches!(c, b'<' | b'>' | b'/' | b'?' | b'\'' | b'"')
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use parserc::Span;

    use crate::reader::lexer::{LexerError, XmLexerState, XmlToken};

    use super::{XmLexer, is_ws};

    #[test]
    fn test_remaining() {
        assert_eq!(XmLexer::from("   ").remaining(), 3);

        assert_eq!(
            XmLexer::from(
                r#"hello

        hello
        "#
            )
            .remaining(),
            29
        );
    }

    #[test]
    fn test_unparsed() {
        assert_eq!(XmLexer::from("   ").unparsed(), "   ");
        assert_eq!(XmLexer::from("   ").unparsed_with(1), " ");
        assert_eq!(XmLexer::from("xx").unparsed_with(3), "xx");
    }

    #[test]
    fn test_state() {
        assert_eq!(XmLexer::from("").state, XmLexerState::Markup);

        let mut lexer = XmLexer::from("");

        lexer.chardata();

        assert_eq!(lexer.state, XmLexerState::CharData);

        lexer.markup();

        assert_eq!(lexer.state, XmLexerState::Markup);
    }

    #[test]
    fn test_is_ws() {
        for c in [b'\x20', b'\x09', b'\x0d', b'\x0a'] {
            assert_eq!(is_ws(c), true);
        }
    }

    #[test]
    fn test_cursor() {
        let mut lexer = XmLexer::new("x\ny", Span::new(10, 3, 2, 3));

        assert_eq!(lexer.next_span(), Span::new(10, 1, 2, 3));

        assert_eq!(lexer.next(), Some(b'x'));
        assert_eq!(lexer.next(), Some(b'\n'));
        assert_eq!(lexer.next_span(), Span::new(12, 1, 3, 1));
        assert_eq!(lexer.next(), Some(b'y'));
        assert_eq!(lexer.next_span(), Span::new(13, 0, 3, 2));

        lexer.seek(Span::new(10, 1, 2, 3));

        assert_eq!(lexer.next(), Some(b'x'));

        catch_unwind(move || lexer.seek(Span::new(9, 1, 2, 3))).expect_err("overflow");
    }

    #[test]
    fn test_next_ws() {
        let mut lexer = XmLexer::from("  ");

        assert_eq!(lexer.next_ws(), Some(XmlToken::WS(Span::new(0, 2, 1, 1))));

        assert_eq!(lexer.next_ws(), None);
    }

    #[test]
    fn test_next_name() {
        assert_eq!(
            XmLexer::from("hell=").next_name(),
            Ok(XmlToken::Name(Span::new(0, 4, 1, 1)))
        );

        assert_eq!(
            XmLexer::from("hell ").next_name(),
            Ok(XmlToken::Name(Span::new(0, 4, 1, 1)))
        );

        assert_eq!(
            XmLexer::from(":hello ").next_name(),
            Ok(XmlToken::Name(Span::new(0, 6, 1, 1)))
        );

        assert_eq!(
            XmLexer::from(".hello ").next_name(),
            Err(LexerError::Name(Span::new(0, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from("-hello ").next_name(),
            Err(LexerError::Name(Span::new(0, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from("<hello ").next_name(),
            Err(LexerError::Name(Span::new(0, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from(" hello ").next_name(),
            Err(LexerError::Name(Span::new(0, 0, 1, 1)))
        );
    }

    #[test]
    fn test_next_chardata() {
        assert_eq!(
            XmLexer::from("hell=").with_chardata().next_chardata(),
            Some(XmlToken::CharData(Span::new(0, 5, 1, 1)))
        );

        assert_eq!(XmLexer::from("<").with_chardata().next_chardata(), None);

        assert_eq!(
            XmLexer::from("hell <").with_chardata().next_chardata(),
            Some(XmlToken::CharData(Span::new(0, 5, 1, 1)))
        );
    }

    #[test]
    fn test_next_cdata() {
        assert_eq!(XmLexer::from("hell=").next_cdata(), Ok(None));
        assert_eq!(
            XmLexer::from("<![CDATA[").next_cdata(),
            Err(LexerError::CData(Span::new(9, 0, 1, 1)))
        );
        assert_eq!(
            XmLexer::from("<![CDATA[\ndfdfd<<<]]]]]").next_cdata(),
            Err(LexerError::CData(Span::new(9, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from("<![CDATA[\ndfdfd<<<]]]]]>").next_cdata(),
            Ok(Some(XmlToken::CData(Span::new(9, 12, 1, 1))))
        );

        assert_eq!(
            XmLexer::from("<![CDATA[ hello  < hll <!--- ]]>").next_cdata(),
            Ok(Some(XmlToken::CData(Span::new(9, 20, 1, 1))))
        );
    }

    #[test]
    fn test_next_comment() {
        assert_eq!(XmLexer::from("hell=").next_comment(), Ok(None));
        assert_eq!(
            XmLexer::from("<!--").next_comment(),
            Err(LexerError::Comment(Span::new(4, 0, 1, 1)))
        );
        assert_eq!(
            XmLexer::from("<!--\ndfdfd<<<-----").next_comment(),
            Err(LexerError::Comment(Span::new(4, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from("<!--\ndfdfd<<<---->").next_comment(),
            Ok(Some(XmlToken::Comment(Span::new(4, 11, 1, 1))))
        );

        assert_eq!(
            XmLexer::from("<!-- hello  < hll <!]]]-->").next_comment(),
            Ok(Some(XmlToken::Comment(Span::new(4, 19, 1, 1))))
        );
    }

    #[test]
    fn test_next_quote_str() {
        assert_eq!(
            XmLexer::from("'--\ndfdfd<<<-----").next_quote_str(),
            Err(LexerError::QuoteStr('\'', Span::new(1, 16, 1, 2)))
        );

        assert_eq!(
            XmLexer::from("'\ndfdfd<<<--'").next_quote_str(),
            Ok(XmlToken::QuoteStr(Span::new(1, 11, 1, 2)))
        );

        assert_eq!(
            XmLexer::from("\" hello  < hll <!]]]-->\"").next_quote_str(),
            Ok(XmlToken::QuoteStr(Span::new(1, 22, 1, 2)))
        );
    }

    #[test]
    fn test_doc_type() {
        assert_eq!(XmLexer::from("<!DOCTYPE").next_doc_type(), None);
        assert_eq!(
            XmLexer::from("<!DOCTYPE>").next_doc_type(),
            Some(XmlToken::DocType(Span::new(9, 0, 1, 1)))
        );

        assert_eq!(
            XmLexer::from(r#"<!DOCTYPE greeting SYSTEM "hello.dtd">"#).next_doc_type(),
            Some(XmlToken::DocType(Span::new(9, 28, 1, 1)))
        );

        assert_eq!(
            XmLexer::from(
                r#"<!DOCTYPE greeting [
                    <!ELEMENT greeting (#PCDATA)>
                ]>
                "#
            )
            .next_doc_type(),
            Some(XmlToken::DocType(Span::new(9, 79, 1, 1)))
        );
    }
}
