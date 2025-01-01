//! Implementations of [`LeaderbotStorage`](crate::leaderbot::LeaderbotStorage).

/// Bot storage keeping data in memory.
#[cfg(feature = "storage-mem")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "storage-mem")))]
pub mod mem {
    use std::collections::HashMap;

    use aoc_leaderboard::aoc::Leaderboard;
    use serde::{Deserialize, Serialize};

    use crate::leaderbot::LeaderbotStorage;

    /// Bot storage that keeps data in memory.
    ///
    /// Can be persisted through [`serde`] if required.
    #[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MemoryLeaderbotStorage {
        previous: HashMap<u64, HashMap<i32, Leaderboard>>,
    }

    impl MemoryLeaderbotStorage {
        /// Creates a new instance without initial data.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl LeaderbotStorage for MemoryLeaderbotStorage {
        type Err = crate::Error;

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
}
