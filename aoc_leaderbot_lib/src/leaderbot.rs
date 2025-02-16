//! Core functionalities of [`aoc_leaderbot`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

pub mod config;
pub mod storage;

use std::collections::HashSet;
use std::future::{ready, Future};

use aoc_leaderboard::aoc::Leaderboard;
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};

use crate::mockable_helpers;

/// Trait that must be implemented to provide the parameters required by the
/// bot to monitor an [Advent of Code] leaderboard.
///
/// [Advent of Code]: https://adventofcode.com/
pub trait Config {
    /// Year for which we want to monitor the leaderboard.
    ///
    /// Defaults to the current year.
    fn year(&self) -> i32 {
        Local::now().year()
    }

    /// ID of the leaderboard to monitor.
    ///
    /// This ID is the last part of the leaderboard's URL, in the form:
    /// `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}`
    ///
    /// It also corresponds to the leaderboard's [`owner_id`](Leaderboard::owner_id).
    fn leaderboard_id(&self) -> u64;

    /// Advent of Code session token.
    ///
    /// This is required to fetch the data of a private leaderboard. A session
    /// token can be obtained from the browser's cookies when visiting the AoC
    /// website. According to the AoC leaderboard API documentation, a session
    /// token lasts about a month.
    fn aoc_session(&self) -> String;
}

/// Trait that must be implemented to persist the data required by the bot
/// in-between every invocation.
#[cfg_attr(test, mockall::automock(type Err=crate::Error;))]
pub trait Storage {
    /// Type of error used by this storage.
    type Err: std::error::Error + Send;

    /// Loads any leaderboard data persisted by a previous bot run.
    ///
    /// If loading was successful but no previous data exists, this method
    /// should return `Ok(None)`.
    fn load_previous(
        &self,
        year: i32,
        leaderboard_id: u64,
    ) -> impl Future<Output = Result<Option<Leaderboard>, Self::Err>> + Send;

    /// Saves the given leaderboard data to storage so that the next bot run
    /// can fetch it using [`load_previous`](Self::load_previous).
    fn save(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send;
}

/// Changes to a leaderboard detected by the bot.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changes {
    /// IDs of new members added to the leaderboard since last run.
    pub new_members: HashSet<u64>,

    /// IDs of members who got new stars since last run.
    pub members_with_new_stars: HashSet<u64>,
}

impl Changes {
    /// Creates a [`Changes`].
    pub fn new(new_members: HashSet<u64>, members_with_new_stars: HashSet<u64>) -> Self {
        Self { new_members, members_with_new_stars }
    }

    /// Creates a [`Changes`] if there are new members and/or members
    /// with new stars, otherwise returns `None`.
    pub fn if_needed(
        new_members: HashSet<u64>,
        members_with_new_stars: HashSet<u64>,
    ) -> Option<Self> {
        if !new_members.is_empty() || !members_with_new_stars.is_empty() {
            Some(Self::new(new_members, members_with_new_stars))
        } else {
            None
        }
    }
}

/// Trait that must be implemented to report changes to the leaderboard.
pub trait Reporter {
    /// Type of error used by this reporter.
    type Err: std::error::Error + Send;

    /// Report changes to the leaderboard.
    ///
    /// The method receives references to both the previous version of the leaderboard,
    /// the current version of the leaderboard, and the lists of changes detected.
    ///
    /// IDs stored in the [`Changes`] point to [leaderboard members] found
    /// in the current version of the leaderboard.
    ///
    /// [leaderboard members]: Leaderboard::members
    fn report_changes(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &Changes,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send;

    /// Report an error that occurred while the bot was running.
    ///
    /// This can be useful to report things to the same channel as
    /// the one where we send the leaderboard changes, so that the
    /// bot owner can fix the issue.
    ///
    /// The default implementation prints the error to `stderr`.
    ///
    /// # Notes
    ///
    /// This method doesn't allow returning an error, because it
    /// will only be called while processing another error.
    /// If an error occurs while sending the error report,
    /// it should simply be ignored internally.
    fn report_error<S>(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error: S,
    ) -> impl Future<Output = ()> + Send
    where
        S: Into<String> + Send,
    {
        let error = error.into();
        eprintln!("Error while looking for changes to leaderboard {leaderboard_id} for year {year}: {error}");
        ready(())
    }
}

/// Runs the bot's core functionality.
///
/// Reads the [`config`], fetches the current leaderboard data, then fetches the previous
/// leaderboard data from [`storage`]. If there was no previous leaderboard (e.g. this is
/// the first run), saves the current leaderboard to storage and exits; otherwise, computes
/// if the leaderboard has new members and/or members who got new stars and calls the
/// [`reporter`] if some diff is found.
///
/// [`config`]: Config
/// [`storage`]: Storage
/// [`reporter`]: Reporter
pub async fn run_bot<C, S, R>(config: &C, storage: &mut S, reporter: &mut R) -> crate::Result<()>
where
    C: Config,
    S: Storage,
    R: Reporter,
    crate::Error: From<<S as Storage>::Err> + From<<R as Reporter>::Err>,
{
    async fn internal_run_bot<S, R>(
        year: i32,
        leaderboard_id: u64,
        aoc_session: String,
        storage: &mut S,
        reporter: &mut R,
    ) -> crate::Result<()>
    where
        S: Storage,
        R: Reporter,
        crate::Error: From<<S as Storage>::Err> + From<<R as Reporter>::Err>,
    {
        let leaderboard =
            mockable_helpers::get_leaderboard(year, leaderboard_id, &aoc_session).await?;

        match storage.load_previous(year, leaderboard_id).await? {
            Some(previous_leaderboard) => {
                if let Some(changes) = detect_changes(&previous_leaderboard, &leaderboard) {
                    reporter
                        .report_changes(
                            year,
                            leaderboard_id,
                            &previous_leaderboard,
                            &leaderboard,
                            &changes,
                        )
                        .await?;
                    storage.save(year, leaderboard_id, &leaderboard).await?;
                }
            },
            None => storage.save(year, leaderboard_id, &leaderboard).await?,
        }

        Ok(())
    }

    let (year, leaderboard_id, aoc_session) =
        (config.year(), config.leaderboard_id(), config.aoc_session());

    match internal_run_bot(year, leaderboard_id, aoc_session, storage, reporter).await {
        Ok(()) => Ok(()),
        Err(err) => {
            reporter
                .report_error(year, leaderboard_id, err.to_string())
                .await;
            Err(err)
        },
    }
}

fn detect_changes(
    previous_leaderboard: &Leaderboard,
    leaderboard: &Leaderboard,
) -> Option<Changes> {
    let new_members = leaderboard
        .members
        .keys()
        .filter(|id| !previous_leaderboard.members.contains_key(id))
        .copied()
        .collect();
    let members_with_new_stars = leaderboard
        .members
        .values()
        .filter(|member| {
            previous_leaderboard
                .members
                .get(&member.id)
                .is_some_and(|prev| prev.stars < member.stars)
        })
        .map(|member| member.id)
        .collect();

    Changes::if_needed(new_members, members_with_new_stars)
}

#[cfg(test)]
#[cfg(all(feature = "config-mem", feature = "storage-mem"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    mod run_bot {
        use std::collections::HashMap;
        use std::future::ready;

        use aoc_leaderboard::aoc::{CompletionDayLevel, LeaderboardMember, PuzzleCompletionInfo};
        use assert_matches::assert_matches;
        use mockall::predicate::eq;
        use serial_test::serial;

        use super::*;
        use crate::leaderbot::config::mem::MemoryConfig;
        use crate::leaderbot::storage::mem::MemoryStorage;

        pub const YEAR: i32 = 2024;
        pub const LEADERBOARD_ID: u64 = 12345;
        pub const AOC_SESSION: &str = "aoc_session";

        const OWNER: u64 = 42;
        const MEMBER_1: u64 = 23;
        const MEMBER_2: u64 = 11;

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct SpiedChanges {
            pub previous_leaderboard: Leaderboard,
            pub leaderboard: Leaderboard,
            pub changes: Changes,
        }

        #[derive(Debug, Default)]
        pub struct SpyReporter {
            pub changes: Vec<(i32, u64, SpiedChanges)>,
            pub errors: Vec<(i32, u64, String)>,
        }

        impl SpyReporter {
            pub fn calls(&self) -> usize {
                self.changes.len() + self.errors.len()
            }

            pub fn called(&self) -> bool {
                self.calls() != 0
            }
        }

        impl Reporter for SpyReporter {
            type Err = crate::Error;

            async fn report_changes(
                &mut self,
                year: i32,
                leaderboard_id: u64,
                previous_leaderboard: &Leaderboard,
                leaderboard: &Leaderboard,
                changes: &Changes,
            ) -> Result<(), Self::Err> {
                self.changes.push((
                    year,
                    leaderboard_id,
                    SpiedChanges {
                        previous_leaderboard: previous_leaderboard.clone(),
                        leaderboard: leaderboard.clone(),
                        changes: changes.clone(),
                    },
                ));

                Ok(())
            }

            async fn report_error<S>(&mut self, year: i32, leaderboard_id: u64, error: S)
            where
                S: Into<String> + Send,
            {
                self.errors.push((year, leaderboard_id, error.into()));
            }
        }

        fn config() -> MemoryConfig {
            MemoryConfig::builder()
                .year(YEAR)
                .leaderboard_id(LEADERBOARD_ID)
                .aoc_session(AOC_SESSION)
                .build()
                .unwrap()
        }

        fn storage() -> MemoryStorage {
            MemoryStorage::new()
        }

        fn spy_reporter() -> SpyReporter {
            SpyReporter::default()
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
                MEMBER_2,
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

        mod without_previous {
            use super::*;

            #[tokio::test]
            #[serial(run_bot)]
            async fn stores_current() {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let ctx = mockable_helpers::get_leaderboard_context();
                ctx.expect().returning(|_, _, _| Ok(base_leaderboard()));

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert!(result.is_ok());
                assert_eq!(storage.len(), 1);
                assert!(!reporter.called());

                let expected = base_leaderboard();
                let actual = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
                assert_eq!(actual, Some(expected));
            }
        }

        mod with_previous {
            use super::*;

            async fn test_previous<N, W>(
                leaderboard: Leaderboard,
                new_members: N,
                members_with_new_stars: W,
            ) where
                N: IntoIterator<Item = u64>,
                W: IntoIterator<Item = u64>,
            {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let base = base_leaderboard();

                storage.save(YEAR, LEADERBOARD_ID, &base).await.unwrap();

                let ctx = mockable_helpers::get_leaderboard_context();
                let returned_leaderboard = leaderboard.clone();
                ctx.expect()
                    .returning(move |_, _, _| Ok(returned_leaderboard.clone()));

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert!(result.is_ok());
                assert_eq!(storage.len(), 1);

                let expected = SpiedChanges {
                    previous_leaderboard: base.clone(),
                    leaderboard: leaderboard.clone(),
                    changes: Changes {
                        new_members: new_members.into_iter().collect(),
                        members_with_new_stars: members_with_new_stars.into_iter().collect(),
                    },
                };
                if expected.changes.new_members.len()
                    + expected.changes.members_with_new_stars.len()
                    != 0
                {
                    assert!(reporter.called());
                    let (actual_year, actual_leaderboard_id, actual) = &reporter.changes[0];
                    assert_eq!(*actual_year, YEAR);
                    assert_eq!(*actual_leaderboard_id, LEADERBOARD_ID);
                    assert_eq!(*actual, expected);
                } else {
                    assert!(!reporter.called())
                }
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn with_no_changes() {
                test_previous(base_leaderboard(), vec![], vec![]).await;
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn with_new_member() {
                test_previous(leaderboard_with_new_member(), vec![MEMBER_2], vec![]).await;
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn with_member_with_new_stars() {
                test_previous(leaderboard_with_member_with_new_stars(), vec![], vec![MEMBER_1])
                    .await;
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn with_both() {
                test_previous(leaderboard_with_both_updates(), vec![MEMBER_2], vec![MEMBER_1])
                    .await;
            }
        }

        mod errors {
            use super::*;

            #[tokio::test]
            #[serial(run_bot)]
            async fn leaderboard_get_error() {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let ctx = mockable_helpers::get_leaderboard_context();
                ctx.expect()
                    .returning(move |_, _, _| Err(aoc_leaderboard::Error::NoAccess));

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert_matches!(
                    result,
                    Err(crate::Error::Leaderboard(aoc_leaderboard::Error::NoAccess))
                );
                assert!(storage.is_empty());
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn load_previous_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let ctx = mockable_helpers::get_leaderboard_context();
                let returned_leaderboard = base_leaderboard();
                ctx.expect()
                    .returning(move |_, _, _| Ok(returned_leaderboard.clone()));

                storage
                    .expect_load_previous()
                    .with(eq(YEAR), eq(LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| {
                        Box::pin(ready(Err(crate::Error::TestLoadPreviousError)))
                    });

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert_matches!(result, Err(crate::Error::TestLoadPreviousError));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(reporter.errors[0], (YEAR, LEADERBOARD_ID, "test".to_string()));
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn report_changes_error() {
                #[derive(Debug, Default)]
                struct MockReporter {
                    pub errors: usize,
                }

                impl Reporter for MockReporter {
                    type Err = crate::Error;

                    async fn report_changes(
                        &mut self,
                        _year: i32,
                        _leaderboard_id: u64,
                        _previous_leaderboard: &Leaderboard,
                        _leaderboard: &Leaderboard,
                        _changes: &Changes,
                    ) -> Result<(), Self::Err> {
                        Err(crate::Error::TestReportChangesError)
                    }

                    async fn report_error<S>(&mut self, _year: i32, _leaderboard_id: u64, _error: S)
                    where
                        S: Into<String> + Send,
                    {
                        self.errors += 1;
                    }
                }

                let config = config();
                let mut storage = storage();
                let mut reporter = MockReporter::default();

                let base = base_leaderboard();
                storage.save(YEAR, LEADERBOARD_ID, &base).await.unwrap();

                let ctx = mockable_helpers::get_leaderboard_context();
                let returned_leaderboard = leaderboard_with_new_member();
                ctx.expect()
                    .returning(move |_, _, _| Ok(returned_leaderboard.clone()));

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert_matches!(result, Err(crate::Error::TestReportChangesError));
                assert_eq!(reporter.errors, 1);
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn save_updated_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let ctx = mockable_helpers::get_leaderboard_context();
                let returned_leaderboard = leaderboard_with_new_member();
                ctx.expect()
                    .returning(move |_, _, _| Ok(returned_leaderboard.clone()));

                storage
                    .expect_load_previous()
                    .with(eq(YEAR), eq(LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| Box::pin(ready(Ok(Some(base_leaderboard())))));
                storage
                    .expect_save()
                    .with(eq(YEAR), eq(LEADERBOARD_ID), eq(leaderboard_with_new_member()))
                    .times(1)
                    .returning(move |_, _, _| {
                        Box::pin(ready(Err(crate::Error::TestSaveUpdatedError)))
                    });

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert_matches!(result, Err(crate::Error::TestSaveUpdatedError));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(reporter.errors[0], (YEAR, LEADERBOARD_ID, "test".to_string()));
            }

            #[tokio::test]
            #[serial(run_bot)]
            async fn save_base_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let ctx = mockable_helpers::get_leaderboard_context();
                let returned_leaderboard = base_leaderboard();
                ctx.expect()
                    .returning(move |_, _, _| Ok(returned_leaderboard.clone()));

                storage
                    .expect_load_previous()
                    .with(eq(YEAR), eq(LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| Box::pin(ready(Ok(None))));
                storage
                    .expect_save()
                    .with(eq(YEAR), eq(LEADERBOARD_ID), eq(base_leaderboard()))
                    .times(1)
                    .returning(move |_, _, _| {
                        Box::pin(ready(Err(crate::Error::TestSaveBaseError)))
                    });

                let result = run_bot(&config, &mut storage, &mut reporter).await;
                assert_matches!(result, Err(crate::Error::TestSaveBaseError));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(reporter.errors[0], (YEAR, LEADERBOARD_ID, "test".to_string()));
            }
        }
    }
}
