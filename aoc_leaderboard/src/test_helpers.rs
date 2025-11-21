//! Test helpers for the [`aoc_leaderbot`] project's crates.
//!
//! Not meant to be used outside the project; no guarantee on API stability.
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{DateTime, Days, TimeZone, Utc};
use reqwest::{Method, StatusCode, header};
use rstest::fixture;
use wiremock::matchers::{header, method, path, path_regex, query_param};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

use crate::aoc::{Leaderboard, LeaderboardCredentials, LeaderboardCredentialsKind};

pub const TEST_YEAR: i32 = 2024;
pub const TEST_LEADERBOARD_ID: u64 = 12345;
pub const TEST_AOC_VIEW_KEY: &str = "aoc_view_key";
pub const TEST_AOC_SESSION: &str = "aoc_session";

pub static TEST_DAY_1: LazyLock<DateTime<Utc>> =
    LazyLock::new(|| Utc.with_ymd_and_hms(TEST_YEAR, 3, 14, 15, 9, 2).unwrap());
pub static TEST_DAY_1_TS: LazyLock<i64> = LazyLock::new(|| TEST_DAY_1.timestamp());
pub static TEST_DAY_2_TS: LazyLock<i64> =
    LazyLock::new(|| (*TEST_DAY_1 + Days::new(1)).timestamp());

pub fn leaderboard_file_path(file_name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "leaderboards", file_name]
        .iter()
        .collect()
}

#[fixture]
pub fn test_leaderboard(#[default("sample_leaderboard.json")] file_name: &str) -> Leaderboard {
    serde_json::from_str(&fs::read_to_string(leaderboard_file_path(file_name)).unwrap()).unwrap()
}

#[fixture]
pub fn test_leaderboard_credentials(
    #[default(LeaderboardCredentialsKind::SessionCookie)] kind: LeaderboardCredentialsKind,
) -> LeaderboardCredentials {
    match kind {
        LeaderboardCredentialsKind::ViewKey => {
            LeaderboardCredentials::ViewKey(TEST_AOC_VIEW_KEY.to_string())
        },
        LeaderboardCredentialsKind::SessionCookie => {
            LeaderboardCredentials::SessionCookie(TEST_AOC_SESSION.to_string())
        },
    }
}

pub fn add_credentials_matchers_to_mock_server(
    mock_builder: MockBuilder,
    credentials: LeaderboardCredentials,
) -> MockBuilder {
    match credentials {
        LeaderboardCredentials::ViewKey(view_key) => {
            mock_builder.and(query_param("view_key", view_key))
        },
        LeaderboardCredentials::SessionCookie(_) => mock_builder
            .and(header(header::COOKIE, credentials.session_cookie_header_value().unwrap())),
    }
}

#[fixture]
pub async fn mock_server_with_leaderboard(
    #[default(test_leaderboard::default())] leaderboard: Leaderboard,
    #[default(test_leaderboard_credentials::default())] credentials: LeaderboardCredentials,
) -> MockServer {
    let mock_server = MockServer::start().await;

    let mut mock_builder = Mock::given(method("GET"))
        .and(path(format!("/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}.json")));
    mock_builder = add_credentials_matchers_to_mock_server(mock_builder, credentials);
    mock_builder
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(leaderboard))
        .mount(&mock_server)
        .await;

    mock_server
}

#[fixture]
pub async fn mock_server_with_inaccessible_leaderboard() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path_regex(
            format!(r"^/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}\.json(\?view_key={TEST_AOC_VIEW_KEY})?$"))
        )
        .respond_with(
            ResponseTemplate::new(StatusCode::BAD_REQUEST)
                .set_body_string("You don't have permission to view that private leaderboard.")
        )
        .mount(&mock_server)
        .await;

    mock_server
}

#[fixture]
pub async fn mock_server_with_leaderboard_with_invalid_json(
    #[default(test_leaderboard_credentials::default())] credentials: LeaderboardCredentials,
) -> MockServer {
    let mock_server = MockServer::start().await;

    let mut mock_builder = Mock::given(method("GET"))
        .and(path(format!("/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}.json")));
    mock_builder = add_credentials_matchers_to_mock_server(mock_builder, credentials);
    mock_builder
        .respond_with(
            ResponseTemplate::new(StatusCode::OK)
                .set_body_string("{\"members\":")
                .insert_header(header::CONTENT_TYPE, "application/json"),
        )
        .mount(&mock_server)
        .await;

    mock_server
}
