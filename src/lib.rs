#![cfg_attr(docsrs, feature(doc_cfg))]

mod traits;
pub use traits::*;

mod impls;

#[cfg(feature = "rayon")]
mod rayon;
