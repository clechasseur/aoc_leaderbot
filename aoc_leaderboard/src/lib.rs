//! A crate with a strongly-typed wrapper for an [Advent of Code] leaderboard,
//! along with ways to fetch them from the AoC website.
//!
//! This crate's API consists essentially of the [`Leaderboard`] type and its
//! related subcomponents. If the `http` feature is enabled, a helper to fetch
//! a leaderboard's data from the Advent of Code website is also provided.
//!
//! [Advent of Code]: https://adventofcode.com/
//! [`Leaderboard`]: aoc::Leaderboard

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(any(nightly_rustc, docsrs), feature(doc_cfg))]

pub mod aoc;
pub mod error;

pub use error::Error;
pub use error::Result;
#[cfg(feature = "http")]
#[doc(hidden)]
pub use reqwest;
