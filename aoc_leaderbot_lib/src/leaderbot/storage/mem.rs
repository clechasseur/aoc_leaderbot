//! Bot storage keeping data in memory.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use aoc_leaderboard::aoc::Leaderboard;
use crate::ErrorKind;
use crate::leaderbot::Storage;

/// Bot storage that keeps data in memory.
///
/// Can be persisted through [`serde`] if required.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStorage {
    previous: HashMap<(i32, u64), (Option<Leaderboard>, Option<ErrorKind>)>,
}

impl MemoryStorage {
    /// Creates a new instance without initial data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of previous leaderboards in storage.
    pub fn len(&self) -> usize {
        self.previous.len()
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
    ) -> Result<(Option<Leaderboard>, Option<ErrorKind>), Self::Err> {
        match self.previous.get(&(year, leaderboard_id)) {
            Some((leaderboard, error_kind)) => {
                Ok((leaderboard.as_ref().cloned(), *error_kind))
            }
            None => Ok((None, None)),
        }
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn save_success(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> Result<(), Self::Err> {
        self.previous.insert((year, leaderboard_id), (Some(leaderboard.clone()), None));
        
        Ok(())
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn save_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error_kind: ErrorKind,
    ) -> Result<(), Self::Err> {
        let (_, prev_err) = self.previous
            .entry((year, leaderboard_id))
            .or_default();
        *prev_err = Some(error_kind);
        
        Ok(())
    }
}
