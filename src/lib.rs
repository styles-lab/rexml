//! A pure rust xml implementation, based-on event stream api.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "reader")]
#[cfg_attr(docsrs, doc(cfg(feature = "reader")))]
pub mod reader;

pub mod types;
