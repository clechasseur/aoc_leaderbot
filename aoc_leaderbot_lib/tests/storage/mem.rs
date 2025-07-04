mod memory_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderboard::test_helpers::{TEST_LEADERBOARD_ID, TEST_YEAR, test_leaderboard};
    use aoc_leaderbot_lib::ErrorKind;
    use aoc_leaderbot_lib::leaderbot::Storage;
    use aoc_leaderbot_lib::leaderbot::storage::mem::MemoryStorage;
    use assert_matches::assert_matches;
    use rstest::rstest;

    mod new {
        use super::*;

        #[test_log::test(tokio::test)]
        async fn new() {
            let storage = MemoryStorage::new();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert!(previous_error.is_none());
        }

        #[test_log::test(tokio::test)]
        async fn default() {
            let storage = MemoryStorage::default();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert!(previous_error.is_none());
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
                .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
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

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert!(previous_error.is_none());

            storage
                .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
                .await
                .unwrap();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert_eq!(previous_leaderboard, Some(expected));
            assert!(previous_error.is_none());

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR - 1, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert!(previous_error.is_none());
        }

        #[rstest]
        #[test_log::test(tokio::test)]
        async fn save_success_and_error(
            #[from(test_leaderboard)] leaderboard: Leaderboard,
            #[from(test_leaderboard)] expected: Leaderboard,
        ) {
            let mut storage = MemoryStorage::new();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert!(previous_error.is_none());

            let error_kind = ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess);
            storage
                .save_error(TEST_YEAR, TEST_LEADERBOARD_ID, error_kind)
                .await
                .unwrap();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert!(previous_leaderboard.is_none());
            assert_matches!(previous_error, Some(previous_err) => {
                assert_eq!(error_kind, previous_err);
            });

            storage
                .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
                .await
                .unwrap();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert_matches!(previous_leaderboard, Some(previous_leaderboard) => {
                assert_eq!(previous_leaderboard, expected);
            });
            assert!(previous_error.is_none());

            storage
                .save_error(TEST_YEAR, TEST_LEADERBOARD_ID, error_kind)
                .await
                .unwrap();

            let (previous_leaderboard, previous_error) = storage
                .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                .await
                .unwrap();
            assert_matches!(previous_leaderboard, Some(previous_leaderboard) => {
                assert_eq!(previous_leaderboard, expected);
            });
            assert_matches!(previous_error, Some(previous_err) => {
                assert_eq!(error_kind, previous_err);
            });
        }
    }
}
