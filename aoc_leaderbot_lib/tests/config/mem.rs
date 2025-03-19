mod memory_config {
    use aoc_leaderboard::test_helpers::{TEST_AOC_SESSION, TEST_LEADERBOARD_ID, TEST_YEAR};
    use aoc_leaderbot_lib::leaderbot::config::mem::MemoryConfig;
    use aoc_leaderbot_lib::leaderbot::Config;

    #[test_log::test]
    fn new() {
        let actual = MemoryConfig::new(TEST_YEAR, TEST_LEADERBOARD_ID, TEST_AOC_SESSION);

        assert_eq!(actual.year(), TEST_YEAR);
        assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
        assert_eq!(actual.aoc_session(), TEST_AOC_SESSION);
    }

    mod builder {
        use std::any::type_name;

        use aoc_leaderbot_lib::Error;
        use assert_matches::assert_matches;
        use chrono::{Datelike, Local};

        use super::*;

        #[test_log::test]
        fn with_all_fields() {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .aoc_session(TEST_AOC_SESSION)
                .build()
                .unwrap();

            assert_eq!(actual.year(), TEST_YEAR);
            assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
            assert_eq!(actual.aoc_session(), TEST_AOC_SESSION);
        }

        #[test_log::test]
        fn with_default_year() {
            let actual = MemoryConfig::builder()
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .aoc_session(TEST_AOC_SESSION)
                .build()
                .unwrap();

            assert_eq!(actual.year(), Local::now().year());
            assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
            assert_eq!(actual.aoc_session(), TEST_AOC_SESSION);
        }

        #[test_log::test]
        fn with_missing_leaderboard_id() {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .aoc_session(TEST_AOC_SESSION)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "leaderboard_id");
            });
        }

        #[test_log::test]
        fn with_missing_aoc_session() {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "aoc_session");
            });
        }
    }
}
