//! [Advent of Code]-related type wrappers.
//!
//! [Advent of Code]: https://adventofcode.com/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;

/// Content of an [Advent of Code] private leaderboard.
///
/// Private leaderboards can be fetched from the Advent of Code website
/// via their API URL: `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}.json`
///
/// Also, in 2025 the Advent of Code website added the capability to generate
/// a read-only link for a leaderboard. This link includes a _view key_ and can be
/// fetched anonymously by appending `?view_key={view_key}` to the above URL.
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
impl Leaderboard {
    /// Fetches this leaderboard's data from the [Advent of Code] website.
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
    #[cfg_attr(not(coverage), tracing::instrument(ret(level = "trace"), err))]
    pub async fn get(
        year: i32,
        id: u64,
        credentials: &LeaderboardCredentials,
    ) -> crate::Result<Self> {
        Self::get_from(Self::http_client()?, "https://adventofcode.com", year, id, credentials)
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
        not(coverage),
        tracing::instrument(skip(http_client), level = "debug", ret(level = "trace"), err)
    )]
    pub async fn get_from<B>(
        http_client: reqwest::Client,
        base: B,
        year: i32,
        id: u64,
        credentials: &LeaderboardCredentials,
    ) -> crate::Result<Self>
    where
        B: AsRef<str> + std::fmt::Debug,
    {
        let mut request = http_client.get(format!(
            "{}/{year}/leaderboard/private/view/{id}.json{}",
            base.as_ref(),
            credentials.view_key_url_suffix()
        ));
        if let Some(cookie_header) = credentials.session_cookie_header_value() {
            request = request.header(reqwest::header::COOKIE, cookie_header);
        }

        let response = request
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);
        match response {
            Ok(response) => Ok(response.json().await?),
            // Note: since 2025, the AoC website actually returns an error when trying to access
            // a leaderboard you don't have access to... but it's a `400 Bad Request` 😭
            Err(err)
                if err
                    .status()
                    .is_some_and(|status| status == reqwest::StatusCode::BAD_REQUEST) =>
            {
                Err(crate::Error::NoAccess)
            },
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
    #[cfg_attr(not(coverage), tracing::instrument(level = "trace", err))]
    pub fn http_client() -> crate::Result<reqwest::Client> {
        Ok(reqwest::Client::builder()
            .user_agent(Self::http_user_agent())
            .build()?)
    }

    #[cfg_attr(not(coverage), tracing::instrument(level = "trace", ret))]
    fn http_user_agent() -> String {
        format!("clechasseur/{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/// Credentials necessary to fetch the content of a leaderboard from
/// the [Advent of Code] website.
///
/// If the leaderboard has a read-only link, it can be fetched using its
/// `view_key`; otherwise, an Advent of Code `session` cookie is required.
///
/// [Advent of Code]: https://adventofcode.com/
#[cfg(feature = "http")]
#[derive(
    veil::Redact,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    gratte::EnumDiscriminants,
    gratte::EnumIs,
)]
#[serde(rename_all = "snake_case")]
#[strum_discriminants(
    name(LeaderboardCredentialsKind),
    derive(Serialize, Deserialize, gratte::EnumIs)
)]
pub enum LeaderboardCredentials {
    /// View key allowing anonymous access to the leaderboard.
    ///
    /// Can be obtained by looking at the end of the leaderboard's
    /// read-only like: `?view_key={view_key}`.
    #[redact(all)]
    ViewKey(String),

    /// `session` cookie allowing authenticated access to the leaderboard.
    ///
    /// Can be fetched from the browser's cookie store when logged into the
    /// [Advent of Code] website.
    ///
    /// [Advent of Code]: https://adventofcode.com/
    #[redact(all)]
    SessionCookie(String),
}

#[cfg(feature = "http")]
impl LeaderboardCredentials {
    /// Leaderboard view key.
    ///
    /// Will return `None` if the credentials do not specify a view key.
    pub fn view_key(&self) -> Option<&str> {
        match self {
            LeaderboardCredentials::ViewKey(key) => Some(key.as_ref()),
            LeaderboardCredentials::SessionCookie(_) => None,
        }
    }

    /// Session cookie allowing access to the leaderboard.
    ///
    /// Will return `None` if the credentials do not specify a session cookie.
    pub fn session_cookie(&self) -> Option<&str> {
        match self {
            LeaderboardCredentials::SessionCookie(cookie) => Some(cookie.as_ref()),
            LeaderboardCredentials::ViewKey(_) => None,
        }
    }

    /// URL suffix to use to specify the leaderboard's view key.
    ///
    /// Will return an empty string if the credentials do not specify a view key.
    pub fn view_key_url_suffix(&self) -> String {
        self.view_key()
            .map(|key| format!("?view_key={key}"))
            .unwrap_or_default()
    }

    /// Value of the `cookie` header to pass when loading the leaderboard data.
    ///
    /// Will return `None` if the credentials do not specify a session cookie.
    pub fn session_cookie_header_value(&self) -> Option<String> {
        self.session_cookie()
            .map(|cookie| format!("session={cookie}"))
    }
}

#[cfg(feature = "http")]
impl PartialEq<LeaderboardCredentialsKind> for LeaderboardCredentials {
    fn eq(&self, other: &LeaderboardCredentialsKind) -> bool {
        LeaderboardCredentialsKind::from(self) == *other
    }
}

#[cfg(feature = "http")]
impl PartialEq<LeaderboardCredentials> for LeaderboardCredentialsKind {
    fn eq(&self, other: &LeaderboardCredentials) -> bool {
        *self == Self::from(other)
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
    ///
    /// Note: in 2025, the global leaderboard was removed, so this value will always
    /// be 0 for years 2025 or later.
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
            TEST_LEADERBOARD_ID, TEST_YEAR, mock_server_with_inaccessible_leaderboard,
            mock_server_with_leaderboard, mock_server_with_leaderboard_with_invalid_json,
            test_leaderboard, test_leaderboard_credentials,
        };

        mod deserialize {
            use super::*;

            #[rstest]
            #[test_log::test]
            fn test_deserialize(#[from(test_leaderboard)] leaderboard: Leaderboard) {
                assert_eq!(leaderboard.year, 2024);
                assert_eq!(leaderboard.members.len(), 8);
                assert!(
                    leaderboard.members[&12345].completion_day_level[&2]
                        .part_2
                        .is_some()
                );
            }
        }

        #[cfg(feature = "http")]
        mod get {
            use assert_matches::assert_matches;
            use reqwest::StatusCode;
            use wiremock::MockServer;

            use super::*;

            async fn get_mock_leaderboard(
                credentials: &LeaderboardCredentials,
                mock_server: &MockServer,
            ) -> crate::Result<Leaderboard> {
                Leaderboard::get_from(
                    Leaderboard::http_client()?,
                    mock_server.uri(),
                    TEST_YEAR,
                    TEST_LEADERBOARD_ID,
                    credentials,
                )
                .await
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn success(
                #[from(test_leaderboard)] expected: Leaderboard,
                #[values(
                    LeaderboardCredentialsKind::ViewKey,
                    LeaderboardCredentialsKind::SessionCookie
                )]
                credentials_kind: LeaderboardCredentialsKind,
                #[from(test_leaderboard_credentials)]
                #[with(credentials_kind)]
                credentials: LeaderboardCredentials,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(expected.clone(), credentials.clone())]
                mock_server: MockServer,
            ) {
                let _ = credentials_kind;

                let actual = get_mock_leaderboard(&credentials, &mock_server).await;
                assert_matches!(actual, Ok(actual) => {
                    assert_eq!(actual, expected);
                });
            }

            mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn no_access(
                    #[values(
                        LeaderboardCredentialsKind::ViewKey,
                        LeaderboardCredentialsKind::SessionCookie
                    )]
                    credentials_kind: LeaderboardCredentialsKind,
                    #[future]
                    #[from(mock_server_with_inaccessible_leaderboard)]
                    mock_server: MockServer,
                ) {
                    let credentials = test_leaderboard_credentials(credentials_kind);

                    let actual = get_mock_leaderboard(&credentials, &mock_server).await;
                    assert_matches!(actual, Err(crate::Error::NoAccess));
                }

                #[rstest]
                #[test_log::test(tokio::test)]
                async fn not_found(
                    #[values(
                        LeaderboardCredentialsKind::ViewKey,
                        LeaderboardCredentialsKind::SessionCookie
                    )]
                    credentials_kind: LeaderboardCredentialsKind,
                ) {
                    let credentials = test_leaderboard_credentials(credentials_kind);
                    let mock_server = MockServer::start().await;

                    let actual = get_mock_leaderboard(&credentials, &mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) => {
                        assert!(err.is_status());
                        assert_eq!(err.status(), Some(StatusCode::NOT_FOUND));
                    });
                }

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn invalid_json(
                    #[values(
                        LeaderboardCredentialsKind::ViewKey,
                        LeaderboardCredentialsKind::SessionCookie
                    )]
                    credentials_kind: LeaderboardCredentialsKind,
                    #[from(test_leaderboard_credentials)]
                    #[with(credentials_kind)]
                    credentials: LeaderboardCredentials,
                    #[future]
                    #[from(mock_server_with_leaderboard_with_invalid_json)]
                    #[with(credentials.clone())]
                    mock_server: MockServer,
                ) {
                    let _ = credentials_kind;

                    let actual = get_mock_leaderboard(&credentials, &mock_server).await;
                    assert_matches!(actual, Err(crate::Error::HttpGet(err)) if err.is_decode());
                }
            }
        }
    }
}
