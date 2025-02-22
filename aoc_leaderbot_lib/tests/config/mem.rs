mod memory_config {
    use aoc_leaderbot_lib::leaderbot::config::mem::MemoryConfig;
    use aoc_leaderbot_lib::leaderbot::Config;
    use aoc_leaderbot_test_helpers::{AOC_SESSION, LEADERBOARD_ID, YEAR};

    #[test_log::test]
    fn new() {
        let actual = MemoryConfig::new(YEAR, LEADERBOARD_ID, AOC_SESSION);

        assert_eq!(actual.year(), YEAR);
        assert_eq!(actual.leaderboard_id(), LEADERBOARD_ID);
        assert_eq!(actual.aoc_session(), AOC_SESSION);
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
                .year(YEAR)
                .leaderboard_id(LEADERBOARD_ID)
                .aoc_session(AOC_SESSION)
                .build()
                .unwrap();

            assert_eq!(actual.year(), YEAR);
            assert_eq!(actual.leaderboard_id(), LEADERBOARD_ID);
            assert_eq!(actual.aoc_session(), AOC_SESSION);
        }

        #[test_log::test]
        fn with_default_year() {
            let actual = MemoryConfig::builder()
                .leaderboard_id(LEADERBOARD_ID)
                .aoc_session(AOC_SESSION)
                .build()
                .unwrap();

            assert_eq!(actual.year(), Local::now().year());
            assert_eq!(actual.leaderboard_id(), LEADERBOARD_ID);
            assert_eq!(actual.aoc_session(), AOC_SESSION);
        }

        #[test_log::test]
        fn with_missing_leaderboard_id() {
            let actual = MemoryConfig::builder()
                .year(YEAR)
                .aoc_session(AOC_SESSION)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "leaderboard_id");
            });
        }

        #[test_log::test]
        fn with_missing_aoc_session() {
            let actual = MemoryConfig::builder()
                .year(YEAR)
                .leaderboard_id(LEADERBOARD_ID)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "aoc_session");
            });
        }
    }
}
