//! A low-level and `no-std` friendly implemenation of the xml parser .

mod errors;
pub use errors::*;

mod name;
pub use name::*;

mod utils;
pub use utils::*;

mod misc;
pub use misc::*;

mod attr;
pub use attr::*;

mod chardata;
pub use chardata::*;

mod doctype;
pub use doctype::*;

mod el;
pub use el::*;

mod reader;
pub use reader::*;
