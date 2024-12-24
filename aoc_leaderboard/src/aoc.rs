//! [Advent of Code]-related type wrappers.
//!
//! [Advent of Code]: https://adventofcode.com/

use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Content of an [Advent of Code] private leaderboard.
///
/// Private leaderboards can be fetched from the [Advent of Code] website
/// via their API URL: `https://adventofcode.com/<year>/leaderboard/private/view/<leaderboard_id>.json`
///
/// Leaderboards exist across all years, but can only be fetched for a specific
/// year at a time.
///
/// [Advent of Code]: https://adventofcode.com/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    /// Year of the event for this leaderboard.
    #[serde(rename = "event")]
    pub year: Year,

    /// ID of the [Advent of Code] user that owns this leaderboard.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    pub owner_id: i64,
}

/// Wrapper for a year of an [Advent of Code] event.
///
/// In a leaderboard's JSON representation, this is actually
/// saved as a string.
///
/// [Advent of Code]: https://adventofcode.com/
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Year(pub i32);

impl Deref for Year {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Year {
    type Err = <i32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl TryFrom<String> for Year {
    type Error = <i32 as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Year> for String {
    fn from(value: Year) -> String {
        value.0.to_string()
    }
}

impl From<i32> for Year {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<Year> for i32 {
    fn from(value: Year) -> i32 {
        value.0
    }
}

/// Information about the stats of a member in an [Advent of Code] [`Leaderboard`].
///
/// [Advent of Code]: https://adventofcode.com/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LeaderboardMember {
    /// Member's username.
    ///
    /// Depending on the user's settings, this can be a full name,
    /// a GitHub username or [`None`], which means the user is anonymous.
    pub name: Option<String>,

    /// Member's user ID.
    ///
    /// User IDs are internal to [Advent of Code], but are consistent
    /// across leaderboards and events.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    pub id: u64,

    /// Number of stars obtained by the user for this year's event.
    pub stars: u32,

    /// Member's score in this year's event, local to a given private leaderboard.
    ///
    /// A member's local score is computed using only the scores of other leaderboard members.
    pub local_score: u64,

    /// Member's score in this year's event in the overall leaderboard.
    pub global_score: u64,

    /// Timestamp representing the moment the member obtained their latest star.
    ///
    /// Can be used to determine if the user has progressed in the event.
    pub last_star_ts: u64,
}

/// Information about the completion of each day of this [Advent of Code] event.
/// 
/// [Advent of Code]: https://adventofcode.com/
pub struct CompletionDayLevels {
    
}

/// Information about the completion of a day in an [Advent of Code] event.
/// 
/// [Advent of Code]: https://adventofcode.com/
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CompletionDayLevel {
    /// Info about the completion of the day's part 1.
    #[serde(rename = "1")]
    pub part_1: PuzzleCompletionInfo,
    
    /// Info about the completion of the day's part 2.
    /// 
    /// Can be [`None`] if user has not yet solved part 2.
    #[serde(rename = "2", default, skip_serializing_if = "Option::is_none")]
    pub part_2: Option<PuzzleCompletionInfo>,
}

/// Information about the completion of an [Advent of Code] puzzle.
///
/// [Advent of Code]: https://adventofcode.com/
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PuzzleCompletionInfo {
    /// Timestamp representing the moment the member obtained the star for this puzzle.
    pub get_star_ts: u64,

    /// Star index. No idea what this means yet. 😊
    pub star_index: u64,
}
