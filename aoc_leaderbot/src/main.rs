//! An [Advent of Code] leaderboard-watching bot.
//!
//! `aoc_leaderbot` is a command-line executable that can fetch the data of an [Advent of Code]
//! private leaderboard and compare it to the version of a previous run. If any changes are
//! detected, the bot reports them to various channels.
//!
//! For more information on installing and using the bot, see the [project README].
//!
//! [Advent of Code]: https://adventofcode.com/
//! [project README]: https://github.com/clechasseur/aoc_leaderbot/tree/main/aoc_leaderbot#readme

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

fn main() {
    println!("Hello, world!");
}
