//! Library implementing an [AWS Lambda] handler version of [`aoc_leaderbot`], a bot that can watch
//! an [Advent of Code] private leaderboard for changes and report them to various channels
//! like Slack.
//!
//! [AWS Lambda]: https://aws.amazon.com/lambda/
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod leaderbot;
