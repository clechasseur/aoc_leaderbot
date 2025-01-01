#![allow(dead_code)]

mod config;
mod storage;
pub(crate) mod test_helpers;

mod leaderbot_config {
    use aoc_leaderbot_lib::leaderbot::LeaderbotConfig;
    use chrono::{Datelike, Local};

    use crate::test_helpers::{AOC_SESSION, LEADERBOARD_ID};

    struct TestLeaderbotConfig;

    impl LeaderbotConfig for TestLeaderbotConfig {
        fn leaderboard_id(&self) -> u64 {
            LEADERBOARD_ID
        }

        fn aoc_session(&self) -> String {
            AOC_SESSION.into()
        }
    }

    #[test]
    fn default_year() {
        let config = TestLeaderbotConfig;

        assert_eq!(config.year(), Local::now().year());
        assert_eq!(config.leaderboard_id(), LEADERBOARD_ID);
        assert_eq!(config.aoc_session(), AOC_SESSION);
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

#[cfg(all(feature = "config-mem", feature = "storage-mem"))]
mod run_bot {
    use std::collections::HashMap;

    use aoc_leaderboard::aoc::{
        CompletionDayLevel, Leaderboard, LeaderboardMember, PuzzleCompletionInfo,
    };
    use aoc_leaderbot_lib::leaderbot::config::mem::MemoryLeaderbotConfig;
    use aoc_leaderbot_lib::leaderbot::storage::mem::MemoryLeaderbotStorage;
    use aoc_leaderbot_lib::leaderbot::{LeaderbotConfig, LeaderbotStorage};
    use chrono::Local;

    use crate::test_helpers::{SpyLeaderbotReporter, AOC_SESSION, LEADERBOARD_ID, YEAR};

    const OWNER: u64 = 42;
    const MEMBER_1: u64 = 23;
    const MEMBER_2: u64 = 11;

    fn config() -> impl LeaderbotConfig + Send {
        MemoryLeaderbotConfig::builder()
            .year(YEAR)
            .leaderboard_id(LEADERBOARD_ID)
            .aoc_session(AOC_SESSION)
            .build()
            .unwrap()
    }

    fn storage() -> impl LeaderbotStorage + Send {
        MemoryLeaderbotStorage::new()
    }

    fn spy_reporter() -> SpyLeaderbotReporter {
        SpyLeaderbotReporter::default()
    }

    fn base_leaderboard() -> Leaderboard {
        Leaderboard {
            year: YEAR,
            owner_id: OWNER,
            day1_ts: Local::now().timestamp(),
            members: {
                let mut members = HashMap::new();

                members.insert(
                    OWNER,
                    LeaderboardMember {
                        name: Some("clechasseur".to_string()),
                        id: OWNER,
                        stars: 0,
                        local_score: 0,
                        global_score: 0,
                        last_star_ts: 0,
                        completion_day_level: HashMap::new(),
                    },
                );
                members.insert(
                    MEMBER_1,
                    LeaderboardMember {
                        name: None,
                        id: MEMBER_1,
                        stars: 2,
                        local_score: 10,
                        global_score: 0,
                        last_star_ts: Local::now().timestamp(),
                        completion_day_level: {
                            let mut completion_day_level = HashMap::new();

                            completion_day_level.insert(
                                1,
                                CompletionDayLevel {
                                    part_1: PuzzleCompletionInfo {
                                        get_star_ts: Local::now().timestamp(),
                                        star_index: 1,
                                    },
                                    part_2: Some(PuzzleCompletionInfo {
                                        get_star_ts: Local::now().timestamp(),
                                        star_index: 2,
                                    }),
                                },
                            );

                            completion_day_level
                        },
                    },
                );

                members
            },
        }
    }

    fn add_member_1_stars(leaderboard: &mut Leaderboard) {
        let member_1 = leaderboard.members.get_mut(&MEMBER_1).unwrap();

        member_1.stars += 1;
        member_1.local_score += 5;
        member_1.last_star_ts = Local::now().timestamp();
        member_1.completion_day_level.insert(
            2,
            CompletionDayLevel {
                part_1: PuzzleCompletionInfo {
                    get_star_ts: Local::now().timestamp(),
                    star_index: 3,
                },
                part_2: None,
            },
        );
    }

    fn leaderboard_with_new_member() -> Leaderboard {
        let mut leaderboard = base_leaderboard();

        leaderboard.members.insert(
            MEMBER_1,
            LeaderboardMember {
                name: None,
                id: MEMBER_2,
                stars: 1,
                local_score: 2,
                global_score: 0,
                last_star_ts: Local::now().timestamp(),
                completion_day_level: {
                    let mut completion_day_level = HashMap::new();

                    completion_day_level.insert(
                        1,
                        CompletionDayLevel {
                            part_1: PuzzleCompletionInfo {
                                get_star_ts: Local::now().timestamp(),
                                star_index: 1,
                            },
                            part_2: None,
                        },
                    );

                    completion_day_level
                },
            },
        );

        leaderboard
    }

    fn leaderboard_with_member_with_new_stars() -> Leaderboard {
        let mut leaderboard = base_leaderboard();

        add_member_1_stars(&mut leaderboard);

        leaderboard
    }

    fn leaderboard_with_both_updates() -> Leaderboard {
        let mut leaderboard = leaderboard_with_new_member();

        add_member_1_stars(&mut leaderboard);

        leaderboard
    }
}
