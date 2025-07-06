//! Library implementing utilities required to deploy the [AWS Lambda] version of [`aoc_leaderbot`]
//! to an AWS account. `aoc_leaderbot` is a bot that can watch an [Advent of Code] private
//! leaderboard for changes and report them to various channels like Slack.
//!
//! This library is not meant to be used outside the installer binary and makes no guarantee
//! on API stability. For more information on installing, see [the project page].
//!
//! [AWS Lambda]: https://aws.amazon.com/lambda/
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [the project page]: https://github.com/clechasseur/aoc_leaderbot

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
