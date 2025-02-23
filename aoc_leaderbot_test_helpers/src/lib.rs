//! Test helpers for the [`aoc_leaderbot`] project's crates.
//!
//! Not meant to be used outside the project; no guarantee on API stability.
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

use std::fs;
use std::path::PathBuf;

use aoc_leaderboard::aoc::Leaderboard;
use reqwest::{header, Method, StatusCode};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const YEAR: i32 = 2024;
pub const LEADERBOARD_ID: u64 = 12345;
pub const AOC_SESSION: &str = "aoc_session";

pub fn leaderboard_file_path(file_name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "leaderboards", file_name]
        .iter()
        .collect()
}

pub fn get_test_leaderboard(file_name: &str) -> Leaderboard {
    serde_json::from_str(&fs::read_to_string(leaderboard_file_path(file_name)).unwrap()).unwrap()
}

pub fn get_sample_leaderboard() -> Leaderboard {
    get_test_leaderboard("sample_leaderboard.json")
}

//noinspection DuplicatedCode
pub async fn get_mock_server_with_leaderboard(leaderboard: Leaderboard) -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json")))
        .and(header(header::COOKIE, format!("session={AOC_SESSION}")))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(leaderboard))
        .mount(&mock_server)
        .await;

    mock_server
}

pub async fn get_mock_server_with_sample_leaderboard() -> MockServer {
    get_mock_server_with_leaderboard(get_sample_leaderboard()).await
}

pub async fn get_mock_server_with_inaccessible_leaderboard() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json")))
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
