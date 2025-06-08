//! Test helpers for the [`aoc_leaderbot`] project's crates.
//!
//! Not meant to be used outside the project; no guarantee on API stability.
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{DateTime, Days, TimeZone, Utc};
use reqwest::{header, Method, StatusCode};
use rstest::fixture;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::aoc::Leaderboard;

pub const TEST_YEAR: i32 = 2024;
pub const TEST_LEADERBOARD_ID: u64 = 12345;
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
pub async fn mock_server_with_leaderboard(
    #[default(test_leaderboard::default())] leaderboard: Leaderboard,
) -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}.json")))
        .and(header(header::COOKIE, format!("session={TEST_AOC_SESSION}")))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(leaderboard))
        .mount(&mock_server)
        .await;

    mock_server
}

#[fixture]
pub async fn mock_server_with_inaccessible_leaderboard() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}.json")))
        .respond_with(ResponseTemplate::new(302).insert_header(header::LOCATION, "/"))
        .mount(&mock_server)
        .await;
    Mock::given(method(Method::GET))
        .and(path("/"))
        .respond_with({
            ResponseTemplate::new(200)
                .insert_header(header::CONTENT_TYPE, "text/html")
                .set_body_string("<html><body>Advent of Code</body></html>")
        })
        .mount(&mock_server)
        .await;

    mock_server
}

#[fixture]
pub async fn mock_server_with_leaderboard_with_invalid_json() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{TEST_YEAR}/leaderboard/private/view/{TEST_LEADERBOARD_ID}.json")))
        .and(header(header::COOKIE, format!("session={TEST_AOC_SESSION}")))
        .respond_with(
            ResponseTemplate::new(StatusCode::OK)
                .set_body_string("{\"members\":")
                .insert_header(header::CONTENT_TYPE, "application/json"),
        )
        .mount(&mock_server)
        .await;

    mock_server
}
