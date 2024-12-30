mod config;
mod storage;
pub(crate) mod test_helpers;

mod leaderbot_config {
    use aoc_leaderbot_lib::leaderbot::LeaderbotConfig;
    use chrono::{Datelike, Local};

    struct TestLeaderbotConfig;

    impl LeaderbotConfig for TestLeaderbotConfig {
        fn leaderboard_id(&self) -> u64 {
            12345
        }

        fn aoc_session(&self) -> String {
            "aoc_session".into()
        }
    }

    #[test]
    fn default_year() {
        let config = TestLeaderbotConfig;

        assert_eq!(config.year(), Local::now().year());
        assert_eq!(config.leaderboard_id(), 12345);
        assert_eq!(config.aoc_session(), "aoc_session");
    }
}

mod leaderbot_changes {
    use std::collections::HashSet;

    use aoc_leaderbot_lib::leaderbot::LeaderbotChanges;

    mod with_changes {
        use assert_matches::assert_matches;

        use super::*;

        #[test]
        fn with_new_members() {
            let new_members = [42].iter().copied().collect();
            let members_with_new_stars = HashSet::new();

            let changes = LeaderbotChanges::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert_eq!(ch.new_members.len(), 1);
                assert!(ch.new_members.contains(&42));
                assert!(ch.members_with_new_stars.is_empty());
            });
        }

        #[test]
        fn with_members_with_new_stars() {
            let new_members = HashSet::new();
            let members_with_new_stars = [23].iter().copied().collect();

            let changes = LeaderbotChanges::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert!(ch.new_members.is_empty());
                assert_eq!(ch.members_with_new_stars.len(), 1);
                assert!(ch.members_with_new_stars.contains(&23));
            });
        }

        #[test]
        fn with_both() {
            let new_members = [42].iter().copied().collect();
            let members_with_new_stars = [23].iter().copied().collect();

            let changes = LeaderbotChanges::if_needed(new_members, members_with_new_stars);
            assert_matches!(changes, Some(ch) => {
                assert_eq!(ch.new_members.len(), 1);
                assert!(ch.new_members.contains(&42));
                assert_eq!(ch.members_with_new_stars.len(), 1);
                assert!(ch.members_with_new_stars.contains(&23));
            });
        }
    }

    #[test]
    fn without_changes() {
        let changes = LeaderbotChanges::if_needed(HashSet::new(), HashSet::new());
        assert!(changes.is_none());
    }
}

// mod run_bot {
//     use aoc_leaderbot_lib::leaderbot::config::mem::MemoryLeaderbotConfig;
//
//     const YEAR: i32 = 2024;
//     const LEADERBOARD_ID: u64 = 12345;
//     const AOC_SESSION: &str = "aoc_session";
//
//     fn config() -> MemoryLeaderbotConfig {
//         MemoryLeaderbotConfig::builder()
//             .year(YEAR)
//             .leaderboard_id(LEADERBOARD_ID)
//             .aoc_session(AOC_SESSION)
//             .build()
//             .unwrap()
//     }
// }
