#[cfg(feature = "storage-mem")]
mod mem {
    use aoc_leaderbot_lib::leaderbot::storage::mem::MemoryStorage;
    use aoc_leaderbot_lib::leaderbot::Storage;

    use crate::test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};

    mod new {
        use super::*;

        #[tokio::test]
        async fn new() {
            let storage = MemoryStorage::new();

            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert!(previous.is_none());
        }

        #[tokio::test]
        async fn default() {
            let storage = MemoryStorage::default();

            let previous = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
            assert!(previous.is_none());
        }
    }

    mod mem_storage_impl {
        use super::*;

        #[tokio::test]
        async fn len_and_is_empty() {
            let mut storage = MemoryStorage::new();

            assert_eq!(storage.len(), 0);
            assert!(storage.is_empty());

            storage
                .save(YEAR, LEADERBOARD_ID, &get_sample_leaderboard())
                .await
                .unwrap();

            assert_eq!(storage.len(), 1);
            assert!(!storage.is_empty());
        }
    }

    mod storage_impl {
        use super::*;

        #[tokio::test]
        async fn load_save() {
            let mut storage = MemoryStorage::new();

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
