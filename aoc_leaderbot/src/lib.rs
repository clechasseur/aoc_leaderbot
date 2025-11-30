//! Library implementing the core of [`aoc_leaderbot`], an [Advent of Code] leaderboard-watching
//! bot.
//!
//! This library is considered internal and makes no guarantee on API stability. For more
//! information on using the bot, see the [project README].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [project README]: https://github.com/clechasseur/aoc_leaderbot/tree/main/aoc_leaderbot#readme

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
