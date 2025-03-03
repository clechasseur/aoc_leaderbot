//! Bot storage keeping data in memory.

use std::collections::HashMap;

use aoc_leaderboard::aoc::Leaderboard;
use serde::{Deserialize, Serialize};

use crate::leaderbot::Storage;

/// Bot storage that keeps data in memory.
///
/// Can be persisted through [`serde`] if required.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStorage {
    previous: HashMap<u64, HashMap<i32, Leaderboard>>,
}

impl MemoryStorage {
    /// Creates a new instance without initial data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of previous leaderboards in storage.
    pub fn len(&self) -> usize {
        self.previous.values().map(HashMap::len).sum()
    }

    /// Checks if there are previous leaderboards in storage.
    pub fn is_empty(&self) -> bool {
        self.previous.is_empty()
    }
}

impl Storage for MemoryStorage {
    type Err = crate::Error;

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn load_previous(
        &self,
        year: i32,
        leaderboard_id: u64,
    ) -> Result<Option<Leaderboard>, Self::Err> {
        Ok(self
            .previous
            .get(&leaderboard_id)
            .and_then(|board_prev| board_prev.get(&year))
            .cloned())
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn save(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> Result<(), Self::Err> {
        self.previous
            .entry(leaderboard_id)
            .or_default()
            .insert(year, leaderboard.clone());

        Ok(())
    }
}
