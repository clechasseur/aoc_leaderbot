//! Implementations of [`LeaderbotStorage`](crate::leaderbot::LeaderbotStorage).

/// Bot storage keeping data in memory.
#[cfg(feature = "storage-mem")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "storage-mem")))]
pub mod mem {
    use aoc_leaderboard::aoc::Leaderboard;
    use serde::{Deserialize, Serialize};

    use crate::leaderbot::LeaderbotStorage;

    /// Bot storage that keeps data in memory.
    ///
    /// Can be persisted through [`serde`] if required.
    #[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MemoryLeaderbotStorage {
        previous: Option<Leaderboard>,
    }

    impl MemoryLeaderbotStorage {
        /// Creates a new instance without initial data.
        pub fn new() -> Self {
            Self::default()
        }

        /// Creates a new instance with the given initial data.
        pub fn with_previous(previous: Leaderboard) -> Self {
            Self { previous: Some(previous) }
        }
    }

    impl LeaderbotStorage for MemoryLeaderbotStorage {
        type Err = crate::Error;

        async fn load_previous(&self) -> Result<Option<Leaderboard>, Self::Err> {
            Ok(self.previous.clone())
        }

        async fn save(&mut self, leaderboard: &Leaderboard) -> Result<(), Self::Err> {
            self.previous = Some(leaderboard.clone());

            Ok(())
        }
    }
}
