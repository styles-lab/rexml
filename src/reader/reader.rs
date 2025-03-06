use parserc::Span;

use crate::{reader::lexer::XmlToken, types::XmlVersion};

use super::lexer::{LexerError, XmLexer, XmlSpan};

/// Error type returns by this module.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReadError {
    #[error("read {0} error: incomplete {1}")]
    Incomplete(ReadKind, XmlSpan),

    #[error("read {0} error: {1}")]
    Tokenizer(ReadKind, LexerError),

    #[error("read {kind} error: expect {expect} {span}")]
    Expect {
        kind: ReadKind,
        expect: ReadKind,
        span: XmlSpan,
    },
}

/// Kind of read entity.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReadKind {
    #[error("`attr`")]
    Attr,
    #[error("`attr`")]
    XmlDecl,
    #[error("`'...' or \"..\"`")]
    QuoteStr,
    #[error("`name`")]
    Name,
    #[error("`=`")]
    Eq,
    #[error("`<?`")]
    PIStart,
    #[error("`xml`")]
    ResrvedXml,
    #[error("`version`")]
    ResrvedVersion,
    #[error("`whitespace`")]
    WS,
}

/// variant for xml node.
#[derive(Debug, PartialEq, Clone)]
pub enum XmlNode {
    StartName(Span),
    Children(Span),
    Empty(Span),
    End(Span),
    Text(Span),
    CData(Span),
    Comment(Span),
    DecType(Span),
    PI {
        name: Span,
        unparsed: Span,
    },
    XmlDecl {
        version: XmlVersion,
        encoding: Option<Span>,
        standalone: Option<bool>,
    },
    Eof,
}

/// Read state used by [`Reader`]
#[derive(Debug, PartialEq, Eq)]
pub enum ReadState {
    XmlDecl,
    MiscBeforeElement,
    RootElement,
    Element,
    MiscAfterElement,
    Eof,
}

/// A xml reader with semantic checker.
pub struct Reader<'a> {
    /// The inner lexer of xml.
    #[allow(unused)]
    lexer: XmLexer<'a>,
    /// The inner read state.
    state: ReadState,

    peek: Option<XmlToken>,
}

impl<'a> Reader<'a> {
    fn next_token(&mut self) -> Result<Option<XmlToken>, LexerError> {
        if let Some(token) = self.peek.take() {
            return Ok(Some(token));
        }

        self.lexer.next_token()
    }

    fn expect_keyword(
        &mut self,
        kw: &'static str,
        kind: ReadKind,
        expect: ReadKind,
    ) -> Result<(), ReadError> {
        match self.next_token() {
            Ok(Some(XmlToken::Name(span))) => {
                if self.lexer.fragement(span) != kw {
                    return Err(ReadError::Expect {
                        kind,
                        expect,
                        span: self.lexer.next_span(),
                    });
                }
            }
            Ok(_) => {
                return Err(ReadError::Expect {
                    kind,
                    expect,
                    span: self.lexer.next_span(),
                });
            }
            Err(err) => return Err(ReadError::Tokenizer(kind, err)),
        }

        Ok(())
    }

    fn exepct_ws(&mut self, kind: ReadKind) -> Result<(), ReadError> {
        match self.next_token() {
            Ok(Some(XmlToken::WS(_))) => {}
            Ok(_) => {
                return Err(ReadError::Expect {
                    kind,
                    expect: ReadKind::WS,
                    span: self.lexer.next_span(),
                });
            }
            Err(err) => return Err(ReadError::Tokenizer(kind, err)),
        }

        Ok(())
    }

    fn expect_token<F, T>(&mut self, kind: ReadKind, expect: ReadKind, f: F) -> Result<T, ReadError>
    where
        F: FnOnce(XmlToken) -> Option<T>,
    {
        match self.next_token() {
            Ok(Some(token)) => {
                if let Some(token) = f(token) {
                    Ok(token)
                } else {
                    Err(ReadError::Expect {
                        kind,
                        expect,
                        span: self.lexer.next_span(),
                    })
                }
            }
            Ok(_) => Err(ReadError::Expect {
                kind,
                expect,
                span: self.lexer.next_span(),
            }),
            Err(err) => Err(ReadError::Tokenizer(kind, err)),
        }
    }

    fn expect_eq(&mut self, kind: ReadKind) -> Result<(), ReadError> {
        #[derive(PartialEq)]
        enum EqState {
            BeforeEq,
            Eq,
            AfterEq,
        }

        let mut state = EqState::BeforeEq;

        loop {
            match self.next_token() {
                Ok(Some(XmlToken::WS(span))) => {
                    if state == EqState::BeforeEq {
                        state = EqState::Eq;
                        continue;
                    }

                    if state == EqState::AfterEq {
                        return Ok(());
                    }

                    return Err(ReadError::Expect {
                        kind,
                        expect: ReadKind::Eq,
                        span,
                    });
                }
                Ok(Some(XmlToken::Eq(_))) => {
                    state = EqState::AfterEq;
                    continue;
                }
                Ok(token) => {
                    if state == EqState::AfterEq {
                        return Ok(());
                    }

                    if let Some(token) = token {
                        return Err(ReadError::Expect {
                            kind,
                            expect: ReadKind::Eq,
                            span: token.span(),
                        });
                    } else {
                        return Err(ReadError::Expect {
                            kind,
                            expect: ReadKind::Eq,
                            span: self.lexer.next_span(),
                        });
                    }
                }
                Err(err) => return Err(ReadError::Tokenizer(kind, err)),
            }
        }
    }

    fn next_xmldecl(&mut self) -> Result<XmlNode, ReadError> {
        self.expect_token(ReadKind::XmlDecl, ReadKind::PIStart, |token| {
            if let XmlToken::PIStart(token) = token {
                Some(token)
            } else {
                None
            }
        })?;

        self.expect_keyword("xml", ReadKind::XmlDecl, ReadKind::ResrvedXml)?;

        self.exepct_ws(ReadKind::WS)?;

        self.expect_keyword("version", ReadKind::XmlDecl, ReadKind::ResrvedVersion)?;

        self.expect_eq(ReadKind::XmlDecl)?;

        let version = self.expect_token(ReadKind::XmlDecl, ReadKind::PIStart, |token| {
            if let XmlToken::QuoteStr(token) = token {
                Some(token)
            } else {
                None
            }
        })?;

        todo!()
    }

    fn next_misc(&mut self) -> Result<Option<XmlNode>, ReadError> {
        todo!()
    }

    fn next_root_element(&mut self) -> Result<XmlNode, ReadError> {
        todo!()
    }

    fn next_element(&mut self) -> Result<XmlNode, ReadError> {
        todo!()
    }
}

impl<'a> Reader<'a> {
    /// Create a new `Reader` isntance with provided `lexer` and `state                                      `
    pub fn new<L>(lexer: L, state: ReadState) -> Self
    where
        XmLexer<'a>: From<L>,
    {
        Self {
            lexer: lexer.into(),
            state,
            peek: None,
        }
    }

    /// Process next xml node.
    pub fn next_node(&mut self) -> Result<Option<XmlNode>, ReadError> {
        loop {
            match self.state {
                ReadState::XmlDecl => return self.next_xmldecl().map(|v| Some(v)),
                ReadState::MiscBeforeElement => {
                    if let Some(node) = self.next_misc()? {
                        return Ok(Some(node));
                    } else {
                        self.state = ReadState::RootElement;
                        continue;
                    }
                }
                ReadState::RootElement => return self.next_root_element().map(|v| Some(v)),
                ReadState::Element => return self.next_element().map(|v| Some(v)),
                ReadState::MiscAfterElement => {
                    if let Some(node) = self.next_misc()? {
                        return Ok(Some(node));
                    } else {
                        self.state = ReadState::Eof;
                        continue;
                    }
                }
                ReadState::Eof => return Ok(None),
            }
        }
    }
}
