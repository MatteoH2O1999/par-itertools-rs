#![cfg_attr(docsrs, feature(doc_cfg))]

//!# Par Itertools
//!
//! Extra iterator adaptors, functions and macros that work in parallel contexts.
//!
//! To extend [Iterator] with methods in this crate, import the [ParItertools] trait:
//! ```
//! use par_itertools::ParItertools;
//! ```
//!
//! Now methods like [`permutations`](`ParItertools::permutations`) are available on all iterators:
//! ```
//! use par_itertools::ParItertools;
//! use rayon::iter::ParallelIterator;
//!
//! let mut permutations = (1..4).permutations::<2>().collect::<Vec<_>>();
//! permutations.sort();
//! let mut expected_permutations = vec![[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]];
//! expected_permutations.sort();
//! assert_eq!(permutations, expected_permutations);
//! ```
//!
//! ## Slice optimizations
//!
//! Some operations can be optimized when working with slices.
//! As such, [ParItertools] works also directly on slices to leverage zero-copy operations.
//!
//! ## Crate features
//!
//! * `rayon`:
//!   * Enabled by default.
//!   * Allows integrations with the [rayon] crate.

mod traits;
pub use traits::*;

mod impls;

#[cfg(feature = "rayon")]
#[path = "rayon/mod.rs"]
mod rayon_impl;
