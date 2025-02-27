mod errors;
pub use errors::*;

mod attr;
mod doctype;
mod misc;
mod pi;
mod xmldecl;

mod reader;
pub use reader::*;
