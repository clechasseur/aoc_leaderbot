//! Custom error type definition.

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP error occurring while getting a [`Leaderboard`]'s data
    /// from the [Advent of Code] website (see [`get`]).
    ///
    /// [`Leaderboard`]: crate::aoc::Leaderboard
    /// [Advent of Code]: https://adventofcode.com/
    /// [`get`]: crate::aoc::Leaderboard::get
    #[cfg(feature = "http")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "http")))]
    #[error("http error: {0}")]
    HttpGet(#[from] reqwest::Error),
}
