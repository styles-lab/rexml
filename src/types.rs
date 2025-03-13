//! Common types shared by `reader` and `writer`.

use std::fmt::Display;

/// Represents the xml version num: 1.0 or 1.1
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum XmlVersion {
    Ver10,
    Ver11,
}

impl Display for XmlVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlVersion::Ver10 => write!(f, "1.0"),
            XmlVersion::Ver11 => write!(f, "1.1"),
        }
    }
}
