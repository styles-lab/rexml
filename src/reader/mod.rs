//! A low-level and `no-std` friendly implemenation of the xml parser .

mod errors;
pub use errors::*;

mod name;
pub use name::*;

mod utils;
pub use utils::*;

mod pi;
pub use pi::*;

mod attr;
pub use attr::*;
