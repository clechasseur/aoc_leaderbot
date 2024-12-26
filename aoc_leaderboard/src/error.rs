//! Custom error type definition.

use thiserror::Error;

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP error occurring while getting a [`Leaderboard`]'s data
    /// from the [Advent of Code] website.
    ///
    /// [`Leaderboard`]: crate::aoc::Leaderboard
    /// [Advent of Code]: https://adventofcode.com/
    #[cfg(feature = "http")]
    #[error("http error: {0}")]
    HttpGet(#[from] reqwest::Error),
}
