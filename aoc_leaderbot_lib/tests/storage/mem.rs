mod memory_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderboard::test_helpers::{test_leaderboard, TEST_LEADERBOARD_ID, TEST_YEAR};
    use aoc_leaderbot_lib::leaderbot::storage::mem::MemoryStorage;
    use aoc_leaderbot_lib::leaderbot::Storage;
    use rstest::rstest;

    mod new {
        use super::*;

        #[test_log::test(tokio::test)]
        async fn new() {
            let storage = MemoryStorage::new();

            let previous = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous.is_none());
        }

        #[test_log::test(tokio::test)]
        async fn default() {
            let storage = MemoryStorage::default();

            let previous = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous.is_none());
        }
    }

    mod mem_storage_impl {
        use super::*;

        #[rstest]
        #[test_log::test(tokio::test)]
        async fn len_and_is_empty(#[from(test_leaderboard)] leaderboard: Leaderboard) {
            let mut storage = MemoryStorage::new();

            assert_eq!(storage.len(), 0);
            assert!(storage.is_empty());

            storage
                .save(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
                .await
                .unwrap();

            assert_eq!(storage.len(), 1);
            assert!(!storage.is_empty());
        }
    }

    mod storage_impl {
        use super::*;

        #[rstest]
        #[test_log::test(tokio::test)]
        async fn load_save(
            #[from(test_leaderboard)] leaderboard: Leaderboard,
            #[from(test_leaderboard)] expected: Leaderboard,
        ) {
            let mut storage = MemoryStorage::new();

            let previous = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous.is_none());

            storage
                .save(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
                .await
                .unwrap();

            let previous = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert_eq!(previous, Some(expected));

            let previous = storage
                .load_previous(TEST_YEAR - 1, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous.is_none());
        }
    }
}
