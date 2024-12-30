use std::fs;
use std::path::PathBuf;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderbot_lib::leaderbot::{LeaderbotChanges, LeaderbotReporter};
use aoc_leaderbot_lib::Error;

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

#[derive(Debug, PartialEq, Eq)]
pub struct SpiedChanges {
    pub previous_leaderboard: Leaderboard,
    pub leaderboard: Leaderboard,
    pub changes: LeaderbotChanges,
}

#[derive(Debug, Default)]
pub struct SpyLeaderbotReporter {
    pub changes: Vec<SpiedChanges>,
    pub errors: Vec<String>,
}

impl LeaderbotReporter for SpyLeaderbotReporter {
    type Err = Error;

    async fn report_changes(
        &mut self,
        previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &LeaderbotChanges,
    ) -> Result<(), Self::Err> {
        self.changes.push(SpiedChanges {
            previous_leaderboard: previous_leaderboard.clone(),
            leaderboard: leaderboard.clone(),
            changes: changes.clone(),
        });

        Ok(())
    }

    async fn report_error<S>(&mut self, error: S)
    where
        S: Into<String> + Send,
    {
        self.errors.push(error.into());
    }
}
