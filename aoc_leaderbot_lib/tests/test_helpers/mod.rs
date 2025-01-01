use std::fs;
use std::path::PathBuf;

use aoc_leaderboard::aoc::Leaderboard;

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
