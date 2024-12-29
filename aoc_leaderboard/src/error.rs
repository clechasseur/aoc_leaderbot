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

    /// Error occurring when getting a [`Leaderboard`]'s data
    /// from the [Advent of Code] website, but the AoC session token
    /// does not have access to that private leaderboard.
    ///
    /// This is a separate error than [`HttpGet`](Self::HttpGet) because
    /// when you do not have access to a private leaderboard, the AoC
    /// website redirects you to the main leaderboard instead of returning
    /// a code like `401 Unauthorized`.
    ///
    /// [`Leaderboard`]: crate::aoc::Leaderboard
    /// [Advent of Code]: https://adventofcode.com/
    #[cfg(feature = "http")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "http")))]
    #[error("session does not have access to this leaderboard")]
    NoAccess,
}
