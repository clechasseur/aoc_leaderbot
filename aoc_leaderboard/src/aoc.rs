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
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "http")))]
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
    pub async fn get<S>(year: i32, id: u64, aoc_session: S) -> crate::Result<Leaderboard>
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
    pub async fn get_from<B, S>(
        http_client: reqwest::Client,
        base: B,
        year: i32,
        id: u64,
        aoc_session: S,
    ) -> crate::Result<Leaderboard>
    where
        B: AsRef<str>,
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
    pub fn http_client() -> crate::Result<reqwest::Client> {
        // When trying to fetch the data of a private leaderboard you do
        // not have access to, the AoC website redirects to the main leaderboard,
        // so we won't follow redirect and consider that an unauthorized error.
        Ok(reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(0))
            .user_agent(Self::http_user_agent())
            .build()?)
    }

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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    mod leaderboard {
        use std::fs;
        use std::path::PathBuf;

        use super::*;

        fn leaderboard_file_path(file_name: &str) -> PathBuf {
            [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "leaderboards", file_name]
                .iter()
                .collect()
        }

        fn get_test_leaderboard(file_name: &str) -> Leaderboard {
            serde_json::from_str(&fs::read_to_string(leaderboard_file_path(file_name)).unwrap())
                .unwrap()
        }

        fn get_sample_leaderboard() -> Leaderboard {
            get_test_leaderboard("sample_leaderboard.json")
        }

        mod deserialize {
            use super::*;

            #[test]
            fn test_deserialize() {
                let leaderboard = get_sample_leaderboard();

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
            use reqwest::{header, StatusCode};
            use wiremock::http::Method;
            use wiremock::matchers::{header, method, path};
            use wiremock::{Mock, MockServer, ResponseTemplate};

            use super::*;

            const YEAR: i32 = 2024;
            const LEADERBOARD_ID: u64 = 12345;
            const AOC_SESSION: &str = "aoc_session";

            //noinspection DuplicatedCode
            async fn get_mock_server_with_leaderboard() -> MockServer {
                let mock_server = MockServer::start().await;

                let leaderboard = get_sample_leaderboard();
                Mock::given(method(Method::GET))
                    .and(path(format!("/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json")))
                    .and(header(header::COOKIE, format!("session={AOC_SESSION}")))
                    .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(leaderboard))
                    .mount(&mock_server)
                    .await;

                mock_server
            }

            async fn get_mock_leaderboard(mock_server: &MockServer) -> crate::Result<Leaderboard> {
                Leaderboard::get_from(
                    Leaderboard::http_client()?,
                    mock_server.uri(),
                    YEAR,
                    LEADERBOARD_ID,
                    AOC_SESSION,
                )
                .await
            }

            #[tokio::test]
            async fn success() {
                let mock_server = get_mock_server_with_leaderboard().await;

                let expected = get_sample_leaderboard();
                let actual = get_mock_leaderboard(&mock_server).await;
                assert_matches!(actual, Ok(actual) if actual == expected);
            }

            mod errors {
                use super::*;

                async fn get_mock_server_with_leaderboard_with_no_access() -> MockServer {
                    let mock_server = MockServer::start().await;

                    // When you try to fetch a leaderboard you don't have access to,
                    // the AoC website redirects to the main leaderboard.
                    Mock::given(method(Method::GET))
                        .and(path(format!(
                            "/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json"
                        )))
                        .respond_with(
                            ResponseTemplate::new(StatusCode::SEE_OTHER)
                                .insert_header(header::LOCATION, format!("/{YEAR}/leaderboard")),
                        )
                        .mount(&mock_server)
                        .await;

                    mock_server
                }

                async fn get_mock_server_with_leaderboard_with_invalid_json() -> MockServer {
                    let mock_server = MockServer::start().await;

                    Mock::given(method(Method::GET))
                        .and(path(format!(
                            "/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json"
                        )))
                        .and(header(header::COOKIE, format!("session={AOC_SESSION}")))
                        .respond_with(
                            ResponseTemplate::new(StatusCode::OK)
                                .set_body_string("{\"members\":")
                                .insert_header(header::CONTENT_TYPE, "application/json"),
                        )
                        .mount(&mock_server)
                        .await;

                    mock_server
                }

                #[tokio::test]
                async fn no_access() {
                    let mock_server = get_mock_server_with_leaderboard_with_no_access().await;

                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::NoAccess));
                }

                #[tokio::test]
                async fn not_found() {
                    let mock_server = MockServer::start().await;

                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) => {
                        assert!(err.is_status());
                        assert_eq!(err.status(), Some(StatusCode::NOT_FOUND));
                    });
                }

                #[tokio::test]
                async fn invalid_json() {
                    let mock_server = get_mock_server_with_leaderboard_with_invalid_json().await;

                    let actual = get_mock_leaderboard(&mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) if err.is_decode());
                }
            }
        }
    }
}
