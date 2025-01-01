use std::fs;
use std::path::PathBuf;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderboard::reqwest::{header, StatusCode};
use aoc_leaderbot_lib::leaderbot::{LeaderbotChanges, LeaderbotReporter};
use aoc_leaderbot_lib::Error;
use wiremock::http::Method;
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

async fn get_mock_server_with_leaderboard(leaderboard: Leaderboard) -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method(Method::GET))
        .and(path(format!("/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json")))
        .and(header(header::COOKIE, format!("session={AOC_SESSION}")))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(leaderboard))
        .mount(&mock_server)
        .await;

    mock_server
}

async fn get_mock_server_with_sample_leaderboard() -> MockServer {
    get_mock_server_with_leaderboard(get_sample_leaderboard()).await
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpiedChanges {
    pub previous_leaderboard: Leaderboard,
    pub leaderboard: Leaderboard,
    pub changes: LeaderbotChanges,
}

#[derive(Debug, Default)]
pub struct SpyLeaderbotReporter {
    pub changes: Vec<(i32, u64, SpiedChanges)>,
    pub errors: Vec<(i32, u64, String)>,
}

impl LeaderbotReporter for SpyLeaderbotReporter {
    type Err = Error;

    async fn report_changes(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &LeaderbotChanges,
    ) -> Result<(), Self::Err> {
        self.changes.push((
            year,
            leaderboard_id,
            SpiedChanges {
                previous_leaderboard: previous_leaderboard.clone(),
                leaderboard: leaderboard.clone(),
                changes: changes.clone(),
            },
        ));

        Ok(())
    }

    async fn report_error<S>(&mut self, year: i32, leaderboard_id: u64, error: S)
    where
        S: Into<String> + Send,
    {
        self.errors.push((year, leaderboard_id, error.into()));
    }
}
