#![allow(dead_code)]

mod config;
mod storage;

mod leaderbot_config {
    use aoc_leaderboard::test_helpers::{TEST_AOC_SESSION, TEST_LEADERBOARD_ID};
    use aoc_leaderbot_lib::leaderbot::Config;
    use chrono::{Datelike, Local};

    struct TestLeaderbotConfig;

    impl Config for TestLeaderbotConfig {
        fn leaderboard_id(&self) -> u64 {
            TEST_LEADERBOARD_ID
        }

        fn aoc_session(&self) -> String {
            TEST_AOC_SESSION.into()
        }
    }

    #[test_log::test]
    fn default_year() {
        let config = TestLeaderbotConfig;

        assert_eq!(config.year(), Local::now().year());
        assert_eq!(config.leaderboard_id(), TEST_LEADERBOARD_ID);
        assert_eq!(config.aoc_session(), TEST_AOC_SESSION);
    }
}

mod leaderbot_changes {
    use std::collections::HashSet;

    use aoc_leaderbot_lib::leaderbot::Changes;

    mod with_changes {
        use assert_matches::assert_matches;

        use super::*;

        #[test_log::test]
        fn with_new_members() {
            let new_members = [42].iter().copied().collect();
            let members_with_new_stars = HashSet::new();

            let changes = Changes::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert_eq!(ch.new_members.len(), 1);
                assert!(ch.new_members.contains(&42));
                assert!(ch.members_with_new_stars.is_empty());
            });
        }

        #[test_log::test]
        fn with_members_with_new_stars() {
            let new_members = HashSet::new();
            let members_with_new_stars = [23].iter().copied().collect();

            let changes = Changes::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert!(ch.new_members.is_empty());
                assert_eq!(ch.members_with_new_stars.len(), 1);
                assert!(ch.members_with_new_stars.contains(&23));
            });
        }

        #[test_log::test]
        fn with_both() {
            let new_members = [42].iter().copied().collect();
            let members_with_new_stars = [23].iter().copied().collect();

            let changes = Changes::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert_eq!(ch.new_members.len(), 1);
                assert!(ch.new_members.contains(&42));
                assert_eq!(ch.members_with_new_stars.len(), 1);
                assert!(ch.members_with_new_stars.contains(&23));
            });
        }
    }

    #[test_log::test]
    fn without_changes() {
        let changes = Changes::if_needed(HashSet::new(), HashSet::new());
        assert!(changes.is_none());
    }
}
