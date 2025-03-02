//! A simple and fast xml document processor implementation based-on [`parserc`] crate.
//!

mod errors;
pub use errors::*;

mod events;
pub use events::*;

mod reader;
pub use reader::*;

mod doctype;
mod element;
mod misc;
mod xmldecl;
