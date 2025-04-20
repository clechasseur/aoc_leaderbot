//! Custom error type definition.

use gratte::{EnumDiscriminants, EnumIs};
use serde::{Deserialize, Serialize};

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error, EnumDiscriminants, EnumIs)]
#[non_exhaustive]
#[strum_discriminants(name(ErrorKind), derive(Serialize, Deserialize, EnumIs), non_exhaustive)]
pub enum Error {
    /// HTTP error occurring while getting a [`Leaderboard`]'s data
    /// from the [Advent of Code] website (see [`get`]).
    ///
    /// [`Leaderboard`]: crate::aoc::Leaderboard
    /// [Advent of Code]: https://adventofcode.com/
    /// [`get`]: crate::aoc::Leaderboard::get
    #[cfg(feature = "http")]
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
    #[error("session does not have access to this leaderboard")]
    NoAccess,
}

impl Error {
    /// Returns `true` if the enum is [`Error::HttpGet`] and the internal [`reqwest::Error`]
    /// matches the given predicate.
    #[cfg(feature = "http")]
    pub fn is_http_get_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&reqwest::Error) -> bool,
    {
        match self {
            Self::HttpGet(reqwest_err) => predicate(reqwest_err),
            _ => false,
        }
    }
}

impl PartialEq<ErrorKind> for Error {
    fn eq(&self, other: &ErrorKind) -> bool {
        ErrorKind::from(self) == *other
    }
}

impl PartialEq<Error> for ErrorKind {
    fn eq(&self, other: &Error) -> bool {
        *self == Self::from(other)
    }
}
