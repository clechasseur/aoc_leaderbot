//! Library implementing the core functionalities of [`aoc_leaderbot`], a bot that can watch
//! an [Advent of Code] private leaderboard for changes and report them to various channels
//! like Slack.
//!
//! This library is mostly internal, so it provides no guarantee on API stability.
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(any(nightly_rustc, docsrs), feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub(crate) mod detail;
pub mod error;
pub mod leaderbot;

pub use error::Error;
pub use error::Result;
