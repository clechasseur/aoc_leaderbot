#[cfg(feature = "storage-mem")]
mod memory_leaderbot_storage {
    use aoc_leaderbot_lib::leaderbot::storage::mem::MemoryLeaderbotStorage;
    use aoc_leaderbot_lib::leaderbot::LeaderbotStorage;

    use crate::test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};

    mod new {
        use super::*;

        #[tokio::test]
        async fn new() {
            let storage = MemoryLeaderbotStorage::new();

            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert!(previous.is_none());
        }

        #[tokio::test]
        async fn default() {
            let storage = MemoryLeaderbotStorage::default();

            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert!(previous.is_none());
        }
    }

    mod storage_impl {
        use super::*;

        #[tokio::test]
        async fn load_save() {
            let mut storage = MemoryLeaderbotStorage::new();

            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert!(previous.is_none());

            let leaderboard = get_sample_leaderboard();
            storage
                .save(YEAR, LEADERBOARD_ID, &leaderboard)
                .await
                .unwrap();

            let expected = get_sample_leaderboard();
            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert_eq!(previous, Some(expected));

            let previous = storage
                .load_previous(YEAR - 1, LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous.is_none());
        }
    }
}
