mod memory_config {
    use aoc_leaderboard::aoc::LeaderboardCredentials;
    use aoc_leaderboard::test_helpers::{
        TEST_LEADERBOARD_ID, TEST_YEAR, test_leaderboard_credentials,
    };
    use aoc_leaderbot_lib::leaderbot::Config;
    use aoc_leaderbot_lib::leaderbot::config::mem::MemoryConfig;
    use rstest::rstest;

    #[rstest]
    #[test_log::test]
    fn new(#[from(test_leaderboard_credentials)] credentials: LeaderboardCredentials) {
        let actual = MemoryConfig::new(TEST_YEAR, TEST_LEADERBOARD_ID, credentials.clone());

        assert_eq!(actual.year(), TEST_YEAR);
        assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
        assert_eq!(actual.credentials(), credentials);
    }

    mod builder {
        use std::any::type_name;

        use aoc_leaderbot_lib::Error;
        use assert_matches::assert_matches;
        use chrono::{Datelike, Local};

        use super::*;

        #[rstest]
        #[test_log::test]
        fn with_all_fields(
            #[from(test_leaderboard_credentials)] credentials: LeaderboardCredentials,
        ) {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .credentials(credentials.clone())
                .build()
                .unwrap();

            assert_eq!(actual.year(), TEST_YEAR);
            assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
            assert_eq!(actual.credentials(), credentials);
        }

        #[rstest]
        #[test_log::test]
        fn with_default_year(
            #[from(test_leaderboard_credentials)] credentials: LeaderboardCredentials,
        ) {
            let actual = MemoryConfig::builder()
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .credentials(credentials.clone())
                .build()
                .unwrap();

            assert_eq!(actual.year(), Local::now().year());
            assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
            assert_eq!(actual.credentials(), credentials);
        }

        #[rstest]
        #[test_log::test]
        fn with_missing_leaderboard_id(
            #[from(test_leaderboard_credentials)] credentials: LeaderboardCredentials,
        ) {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .credentials(credentials)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "leaderboard_id");
            });
        }

        #[test_log::test]
        fn with_missing_credentials() {
            let actual = MemoryConfig::builder()
                .year(TEST_YEAR)
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .build();

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<MemoryConfig>());
                assert_eq!(field, "credentials");
            });
        }
    }
}
