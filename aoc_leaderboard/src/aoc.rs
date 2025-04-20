//! [Advent of Code]-related type wrappers.
//!
//! [Advent of Code]: https://adventofcode.com/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

/// Content of an [Advent of Code] private leaderboard.
///
/// Private leaderboards can be fetched from the Advent of Code website
/// via their API URL: `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}.json`
///
/// Leaderboards exist across all years, but can only be fetched for a specific
/// year at a time.
///
/// [Advent of Code]: https://adventofcode.com/
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leaderboard {
    /// Year of the event for this leaderboard.
    #[serde(rename = "event")]
    #[serde_as(as = "DisplayFromStr")]
    pub year: i32,

    /// ID of the [Advent of Code] user that owns this leaderboard.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    pub owner_id: u64,

    /// Possibly the timestamp representing when the day 1 puzzles were completed
    /// by members of this leaderboard? Not sure. 🤔
    #[serde(default)]
    pub day1_ts: i64,

    /// Members of this leaderboard.
    #[serde(default)]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub members: HashMap<u64, LeaderboardMember>,
}

#[cfg(feature = "http")]
#[cfg_attr(use_doc_cfg, doc(cfg(feature = "http")))]
impl Leaderboard {
    /// Fetches this leaderboard's data from the [Advent of Code] website.
    ///
    /// In order to fetch a private leaderboard's data, you must provide
    /// your Advent of Code `session` cookie, which can be fetched from the
    /// browser's cookie store when logged into the website.
    ///
    /// # Warning
    ///
    /// The Advent of Code private leaderboard page (which can be visited
    /// at `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}`)
    /// mentions that you should not fetch a leaderboard's data through this API
    /// more than once every **15 minutes**, so please be mindful not to
    /// overuse this method.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg_attr(
        not(coverage_nightly),
        tracing::instrument(skip(aoc_session), ret(level = "trace"), err)
    )]
    pub async fn get<S>(year: i32, id: u64, aoc_session: S) -> crate::Result<Self>
    where
        S: AsRef<str>,
    {
        Self::get_from(Self::http_client()?, "https://adventofcode.com", year, id, aoc_session)
            .await
    }

    /// Fetches this leaderboard's data from the [Advent of Code] website
    /// using the provided http client and base website URL.
    ///
    /// In general, this method shouldn't be used directly; instead, use [`get`].
    /// See that method's documentation for more details.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    /// [`get`]: Self::get
    #[cfg_attr(
        not(coverage_nightly),
        tracing::instrument(
            skip(http_client, aoc_session),
            level = "debug",
            ret(level = "trace"),
            err
        )
    )]
    pub async fn get_from<B, S>(
        http_client: reqwest::Client,
        base: B,
        year: i32,
        id: u64,
        aoc_session: S,
    ) -> crate::Result<Self>
    where
        B: AsRef<str> + std::fmt::Debug,
        S: AsRef<str>,
    {
        let response = http_client
            .get(format!("{}/{year}/leaderboard/private/view/{id}.json", base.as_ref()))
            .header(reqwest::header::COOKIE, format!("session={}", aoc_session.as_ref()))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);
        match response {
            Ok(response) => Ok(response.json().await?),
            Err(err) if err.is_redirect() => Err(crate::Error::NoAccess),
            Err(err) => Err(err.into()),
        }
    }

    /// Returns an HTTP [`Client`](reqwest::Client) that can be used to
    /// fetch data from the [Advent of Code] website.
    ///
    /// In general, this method shouldn't be used directly; instead, use [`get`],
    /// which creates the HTTP client automatically.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    /// [`get`]: Self::get
    #[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace", err))]
    pub fn http_client() -> crate::Result<reqwest::Client> {
        // When trying to fetch the data of a private leaderboard you do
        // not have access to, the AoC website redirects to the main leaderboard,
        // so we won't follow redirect and consider that an unauthorized error.
        Ok(reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(0))
            .user_agent(Self::http_user_agent())
            .build()?)
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace", ret))]
    fn http_user_agent() -> String {
        format!("clechasseur/{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/// Information about the stats of a member in an [Advent of Code] [`Leaderboard`].
///
/// [Advent of Code]: https://adventofcode.com/
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub stars: u32,

    /// Member's score in this year's event, local to a given private leaderboard.
    ///
    /// A member's local score is computed using only the scores of other leaderboard members.
    #[serde(default)]
    pub local_score: u64,

    /// Member's score in this year's event in the overall leaderboard.
    #[serde(default)]
    pub global_score: u64,

    /// Timestamp representing the moment the member obtained their latest star.
    ///
    /// Can be used to determine if the user has progressed in the event.
    #[serde(default)]
    pub last_star_ts: i64,

    /// Information about completed puzzles for the event.
    #[serde(default)]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub completion_day_level: HashMap<u32, CompletionDayLevel>,
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
    pub get_star_ts: i64,

    /// Star index. No idea what this means yet. 🤔
    #[serde(default)]
    pub star_index: u64,
}

#[cfg(all(test, feature = "__test_helpers"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    mod leaderboard {
        use rstest::rstest;

        use super::*;
        use crate::test_helpers::{
            mock_server_with_inaccessible_leaderboard, mock_server_with_leaderboard,
            mock_server_with_leaderboard_with_invalid_json, test_leaderboard, TEST_AOC_SESSION,
            TEST_LEADERBOARD_ID, TEST_YEAR,
        };

        mod deserialize {
            use super::*;

            #[rstest]
            #[test_log::test]
            fn test_deserialize(#[from(test_leaderboard)] leaderboard: Leaderboard) {
                assert_eq!(leaderboard.year, 2024);
                assert_eq!(leaderboard.members.len(), 8);
                assert!(leaderboard.members[&12345].completion_day_level[&2]
                    .part_2
                    .is_some());
            }
        }

        #[cfg(feature = "http")]
        mod get {
            use assert_matches::assert_matches;
            use reqwest::StatusCode;
            use wiremock::MockServer;

            use super::*;

            async fn get_mock_leaderboard(mock_server: &MockServer) -> crate::Result<Leaderboard> {
                Leaderboard::get_from(
                    Leaderboard::http_client()?,
                    mock_server.uri(),
                    TEST_YEAR,
                    TEST_LEADERBOARD_ID,
                    TEST_AOC_SESSION,
                )
                .await
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn success(
                #[from(test_leaderboard)] expected: Leaderboard,
                #[future]
                #[from(mock_server_with_leaderboard)]
                mock_server: MockServer,
            ) {
                let actual = get_mock_leaderboard(&mock_server).await;
                assert_matches!(actual, Ok(actual) if actual == expected);
            }

            mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn no_access(
                    #[future]
                    #[from(mock_server_with_inaccessible_leaderboard)]
                    mock_server: MockServer,
                ) {
                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::NoAccess));
                }

                #[test_log::test(tokio::test)]
                async fn not_found() {
                    let mock_server = MockServer::start().await;

                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) => {
                        assert!(err.is_status());
                        assert_eq!(err.status(), Some(StatusCode::NOT_FOUND));
                    });
                }

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn invalid_json(
                    #[future]
                    #[from(mock_server_with_leaderboard_with_invalid_json)]
                    mock_server: MockServer,
                ) {
                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) if err.is_decode());
                }
            }
        }
    }
}
