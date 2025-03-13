//! A pure rust xml implementation, based-on event stream api.
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod types;

#[cfg(feature = "reader")]
#[cfg_attr(docsrs, doc(cfg(feature = "reader")))]
pub mod reader;

#[cfg(feature = "writer")]
#[cfg_attr(docsrs, doc(cfg(feature = "writer")))]
pub mod writer;
