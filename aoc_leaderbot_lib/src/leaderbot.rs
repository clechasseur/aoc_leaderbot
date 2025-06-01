//! Core functionalities of [`aoc_leaderbot`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

pub mod config;
pub mod storage;

use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::future::{ready, Future};

use anyhow::anyhow;
use aoc_leaderboard::aoc::Leaderboard;
use chrono::{Datelike, Local};
use gratte::IntoDiscriminant;
use serde::{Deserialize, Serialize};

use crate::error::{ReporterError, StorageError};

/// Trait that must be implemented to provide the parameters required by the
/// bot to monitor an [Advent of Code] leaderboard.
///
/// [Advent of Code]: https://adventofcode.com/
pub trait Config {
    /// Year for which we want to monitor the leaderboard.
    ///
    /// Defaults to the current year.
    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), level = "trace", ret))]
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
    type Err: Error + Send;

    /// Loads any leaderboard data persisted by a previous bot run as well
    /// as information about any error that occurred during the last run.
    ///
    /// If load is successful, returns a tuple with two elements:
    /// 1. The last leaderboard saved via a call to [`save_success`]. Note that
    ///    this will be returned even if the last run resulted in an error.
    /// 2. The kind of error that occurred during the last bot run, if any,
    ///    as passed to [`save_error`].
    ///
    /// [`save_success`]: Self::save_success
    /// [`save_error`]: Self::save_error
    fn load_previous(
        &self,
        year: i32,
        leaderboard_id: u64,
    ) -> impl Future<Output = Result<(Option<Leaderboard>, Option<crate::ErrorKind>), Self::Err>> + Send;

    /// Saves the result of a successful bot run: the latest version of the [`Leaderboard`].
    ///
    /// If the previous bot run resulted in an error persisted via [`save_error`],
    /// that last error should be cleared.
    ///
    /// [`save_error`]: Self::save_error
    fn save_success(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send;

    /// Saves the result of a failed bot run: the kind of error encountered.
    ///
    /// If there exists data about the latest version of the [`Leaderboard`],
    /// as saved by a previous call to [`save_success`], it should **not** be
    /// cleared, since it must be returned if [`load_previous`] is called.
    ///
    /// [`save_success`]: Self::save_success
    /// [`load_previous`]: Self::load_previous
    fn save_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error_kind: crate::ErrorKind,
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
    /// Returns a [`Changes`] with the given new/updated members.
    #[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace"))]
    pub fn new(new_members: HashSet<u64>, members_with_new_stars: HashSet<u64>) -> Self {
        Self { new_members, members_with_new_stars }
    }

    /// Returns a [`Changes`] if there are new members and/or members
    /// with new stars, otherwise returns `None`.
    #[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace", ret))]
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
    type Err: Error + Send;

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
    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self)))]
    fn report_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error: &crate::Error,
    ) -> impl Future<Output = ()> + Send {
        eprintln!("Error while looking for changes to leaderboard {leaderboard_id} for year {year}: {error}");
        ready(())
    }
}

/// Output returned by the [`run_bot`] function. Contains the bot's output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotOutput {
    /// Year for which the bot was run.
    pub year: i32,

    /// ID of leaderboard checked by the bot.
    pub leaderboard_id: u64,

    /// Leaderboard data from previous run, if any.
    ///
    /// If this was the first bot run, will be set to `None`.
    pub previous_leaderboard: Option<Leaderboard>,

    /// Current leaderboard data.
    pub leaderboard: Leaderboard,

    /// Changes detected, if any.
    pub changes: Option<Changes>,
}

/// Runs the bot's core functionality.
///
/// Reads the [`config`], fetches the previous leaderboard data from [`storage`], then
/// fetches current leaderboard data via [`Leaderboard::get`]. If there was no previous
/// leaderboard (e.g. this is the first run), saves the current leaderboard to storage
/// and exits; otherwise, computes if the leaderboard has new members and/or members who
/// got new stars and calls the [`reporter`] if some diff is found.
///
/// If the `dry_run` parameter is set to `true`, then the bot will fetch data and compute
/// changes but will not persist or report them.
///
/// [`config`]: Config
/// [`storage`]: Storage
/// [`reporter`]: Reporter
#[cfg_attr(coverage_nightly, coverage(off))]
#[tracing::instrument(skip(config, storage, reporter), ret, err)]
pub async fn run_bot<C, S, R>(
    config: &C,
    storage: &mut S,
    reporter: &mut R,
    dry_run: bool,
) -> crate::Result<BotOutput>
where
    C: Config,
    S: Storage,
    <S as Storage>::Err: Error + Sync + 'static,
    R: Reporter,
    <R as Reporter>::Err: Error + Sync + 'static,
{
    run_bot_from(None::<String>, config, storage, reporter, dry_run).await
}

/// Runs the bot's core functionality, using the given base Advent of Code URL
/// (or the default, `https://adventofcode.com`, if not provided).
///
/// This function is mostly exposed for testing; you should use [`run_bot`] instead.
#[cfg_attr(
    not(coverage_nightly),
    tracing::instrument(skip(config, storage, reporter), level = "debug", ret, err)
)]
pub async fn run_bot_from<B, C, S, R>(
    advent_of_code_base: Option<B>,
    config: &C,
    storage: &mut S,
    reporter: &mut R,
    dry_run: bool,
) -> crate::Result<BotOutput>
where
    B: AsRef<str> + Debug,
    C: Config,
    S: Storage,
    <S as Storage>::Err: Error + Sync + 'static,
    R: Reporter,
    <R as Reporter>::Err: Error + Sync + 'static,
{
    async fn get_leaderboard_and_changes<B, R>(
        advent_of_code_base: Option<B>,
        year: i32,
        leaderboard_id: u64,
        aoc_session: &str,
        previous_leaderboard: Option<Leaderboard>,
        reporter: &mut R,
        dry_run: bool,
    ) -> crate::Result<BotOutput, crate::Error>
    where
        B: AsRef<str> + Debug,
        R: Reporter,
        <R as Reporter>::Err: Error + Sync + 'static,
    {
        #[cfg_attr(coverage_nightly, coverage(off))]
        async fn get_leaderboard<B>(
            advent_of_code_base: Option<B>,
            year: i32,
            leaderboard_id: u64,
            aoc_session: &str,
        ) -> crate::Result<Leaderboard>
        where
            B: AsRef<str> + Debug,
        {
            Ok(match advent_of_code_base {
                Some(base) => {
                    Leaderboard::get_from(
                        Leaderboard::http_client()?,
                        base,
                        year,
                        leaderboard_id,
                        aoc_session,
                    )
                    .await?
                },
                None => Leaderboard::get(year, leaderboard_id, aoc_session).await?,
            })
        }

        let leaderboard =
            get_leaderboard(advent_of_code_base, year, leaderboard_id, aoc_session).await?;

        let changes = detect_changes(previous_leaderboard.as_ref(), &leaderboard);
        let output = BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard, changes };

        match (&output.previous_leaderboard, &output.changes) {
            (Some(previous_leaderboard), Some(changes)) if !dry_run => {
                reporter
                    .report_changes(
                        year,
                        leaderboard_id,
                        previous_leaderboard,
                        &output.leaderboard,
                        changes,
                    )
                    .await
                    .map_err(|err| ReporterError::ReportChanges(anyhow!(err)))?;
            },
            _ => (),
        }

        Ok(output)
    }

    let (year, leaderboard_id, aoc_session) =
        (config.year(), config.leaderboard_id(), config.aoc_session());

    let previous_result = storage.load_previous(year, leaderboard_id).await;
    let (mut output_result, previous_error) = match previous_result {
        Ok((previous_leaderboard, previous_error)) => {
            let output_result = get_leaderboard_and_changes(
                advent_of_code_base,
                year,
                leaderboard_id,
                &aoc_session,
                previous_leaderboard,
                reporter,
                dry_run,
            )
            .await;
            (output_result, previous_error)
        },
        Err(err) => (Err(StorageError::LoadPrevious(anyhow!(err)).into()), None),
    };

    output_result = match output_result {
        Ok(output) if !dry_run => {
            match storage
                .save_success(year, leaderboard_id, &output.leaderboard)
                .await
            {
                Ok(()) => Ok(output),
                Err(err) => Err(StorageError::Save(anyhow!(err)).into()),
            }
        },
        output_result => output_result,
    };

    match output_result {
        Err(err) if previous_error.is_some_and(|err_kind| err_kind == err.discriminant()) => {
            // An error occurred, but it's the same kind of error reported previously; don't
            // spam the reporter when this happens, simply avoid reporting the same error twice.
            Err(err)
        },
        Err(err) if !dry_run => {
            reporter.report_error(year, leaderboard_id, &err).await;

            if let Err(storage_err) = storage
                .save_error(year, leaderboard_id, (&err).into())
                .await
            {
                // An error occurred while doing the bot run, and an error also occurred
                // while trying to persist information about the last error. ¯\_(ツ)_/¯
                let storage_err = StorageError::Save(anyhow!(storage_err)).into();
                reporter
                    .report_error(year, leaderboard_id, &storage_err)
                    .await;
            }

            Err(err)
        },
        output_result => output_result,
    }
}

#[cfg_attr(not(coverage_nightly), tracing::instrument(ret))]
fn detect_changes(
    previous_leaderboard: Option<&Leaderboard>,
    leaderboard: &Leaderboard,
) -> Option<Changes> {
    match previous_leaderboard {
        Some(previous_leaderboard) => {
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
        },
        None => None,
    }
}

#[cfg(test)]
#[cfg(all(feature = "config-mem", feature = "storage-mem"))]
#[cfg_attr(coverage_nightly, coverage(off))]
// noinspection DuplicatedCode
mod tests {
    use aoc_leaderboard::test_helpers::{
        mock_server_with_inaccessible_leaderboard, mock_server_with_leaderboard, test_leaderboard,
        TEST_AOC_SESSION, TEST_LEADERBOARD_ID, TEST_YEAR,
    };
    use rstest::{fixture, rstest};

    use super::*;

    mod reporter {
        use super::*;

        #[fixture]
        fn reporter() -> impl Reporter {
            struct TestReporter;

            impl Reporter for TestReporter {
                type Err = crate::Error;

                async fn report_changes(
                    &mut self,
                    year: i32,
                    leaderboard_id: u64,
                    _previous_leaderboard: &Leaderboard,
                    _leaderboard: &Leaderboard,
                    changes: &Changes,
                ) -> Result<(), Self::Err> {
                    println!(
                        "Leaderboard {leaderboard_id} (for year {year}) has changed: {} new members added, {} members got new stars",
                        changes.new_members.len(),
                        changes.members_with_new_stars.len()
                    );
                    Ok(())
                }
            }

            TestReporter
        }

        #[rstest]
        #[tokio::test]
        async fn default_impl_works(
            #[from(test_leaderboard)] leaderboard: Leaderboard,
            mut reporter: impl Reporter,
        ) {
            let changes = Changes::new([42, 23].into(), [11, 7].into());

            reporter
                .report_changes(
                    TEST_YEAR,
                    TEST_LEADERBOARD_ID,
                    &leaderboard,
                    &leaderboard,
                    &changes,
                )
                .await
                .unwrap();

            reporter
                .report_error(
                    TEST_YEAR,
                    TEST_LEADERBOARD_ID,
                    &crate::Error::TestErrorWithMessage("broken pipe".into()),
                )
                .await;
        }
    }

    mod run_bot {
        use std::collections::HashMap;
        use std::future::ready;

        use aoc_leaderboard::aoc::{CompletionDayLevel, LeaderboardMember, PuzzleCompletionInfo};
        use aoc_leaderboard::wiremock::MockServer;
        use assert_matches::assert_matches;
        use mockall::predicate::eq;

        use super::*;
        use crate::error::{ReporterErrorKind, StorageErrorKind};
        use crate::leaderbot::config::mem::MemoryConfig;
        use crate::leaderbot::storage::mem::MemoryStorage;

        const OWNER: u64 = 42;
        const MEMBER_1: u64 = 23;
        const MEMBER_2: u64 = 11;

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct SpiedChanges {
            pub previous_leaderboard: Leaderboard,
            pub leaderboard: Leaderboard,
            pub changes: Option<Changes>,
        }

        impl SpiedChanges {
            pub fn has_changes(&self) -> bool {
                self.changes.as_ref().is_some_and(|changes| {
                    !changes.new_members.is_empty() || !changes.members_with_new_stars.is_empty()
                })
            }
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
                        changes: Some(changes.clone()),
                    },
                ));

                Ok(())
            }

            async fn report_error(&mut self, year: i32, leaderboard_id: u64, error: &crate::Error) {
                self.errors.push((year, leaderboard_id, error.to_string()));
            }
        }

        #[fixture]
        fn config() -> MemoryConfig {
            MemoryConfig::builder()
                .year(TEST_YEAR)
                .leaderboard_id(TEST_LEADERBOARD_ID)
                .aoc_session(TEST_AOC_SESSION)
                .build()
                .unwrap()
        }

        #[fixture]
        fn storage() -> MemoryStorage {
            MemoryStorage::new()
        }

        #[fixture]
        fn reporter() -> SpyReporter {
            SpyReporter::default()
        }

        #[fixture]
        fn base_leaderboard() -> Leaderboard {
            Leaderboard {
                year: TEST_YEAR,
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

        #[fixture]
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

        #[fixture]
        fn leaderboard_with_member_with_new_stars(
            #[from(base_leaderboard)] mut leaderboard: Leaderboard,
        ) -> Leaderboard {
            add_member_1_stars(&mut leaderboard);

            leaderboard
        }

        #[fixture]
        fn leaderboard_with_both_updates(
            #[from(leaderboard_with_new_member)] mut leaderboard: Leaderboard,
        ) -> Leaderboard {
            add_member_1_stars(&mut leaderboard);

            leaderboard
        }

        mod without_previous {
            use super::*;

            #[rstest]
            #[case::stores_current(false)]
            #[case::dry_run_does_not_store_current(true)]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn and(
                config: MemoryConfig,
                mut storage: MemoryStorage,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(base_leaderboard::default())]
                mock_server: MockServer,
                #[case] dry_run: bool,
                #[from(base_leaderboard)] expected: Leaderboard,
            ) {
                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    dry_run,
                )
                .await;
                assert_matches!(result, Ok(BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard, changes }) => {
                    assert_eq!(year, TEST_YEAR);
                    assert_eq!(leaderboard_id, TEST_LEADERBOARD_ID);
                    assert!(previous_leaderboard.is_none());
                    assert_eq!(leaderboard, expected);
                    assert!(changes.is_none());
                });
                assert_eq!(storage.len(), if dry_run { 0 } else { 1 });
                assert!(!reporter.called());

                let (actual_leaderboard, actual_err) = storage
                    .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                    .await
                    .unwrap();
                assert!(actual_err.is_none());
                assert_eq!(actual_leaderboard, if dry_run { None } else { Some(expected) });
            }
        }

        #[allow(clippy::too_many_arguments)]
        mod with_previous {
            use super::*;

            #[rstest]
            #[case::no_changes(base_leaderboard::default(), vec![], vec![])]
            #[case::new_member(leaderboard_with_new_member::default(), vec![MEMBER_2], vec![])]
            #[case::member_with_new_stars(leaderboard_with_member_with_new_stars::default(), vec![], vec![MEMBER_1])]
            #[case::both_updates(leaderboard_with_both_updates::default(), vec![MEMBER_2], vec![MEMBER_1])]
            #[test_log::test(tokio::test)]
            async fn and(
                config: MemoryConfig,
                mut storage: MemoryStorage,
                mut reporter: SpyReporter,
                #[from(base_leaderboard)] base: Leaderboard,
                #[case] leaderboard: Leaderboard,
                #[case] expected_new_members: Vec<u64>,
                #[case] expected_members_with_new_stars: Vec<u64>,
                #[values(false, true)] dry_run: bool,
            ) {
                storage
                    .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &base)
                    .await
                    .unwrap();

                let expected = SpiedChanges {
                    previous_leaderboard: base.clone(),
                    leaderboard: leaderboard.clone(),
                    changes: Changes::if_needed(
                        expected_new_members.into_iter().collect(),
                        expected_members_with_new_stars.into_iter().collect(),
                    ),
                };

                let mock_server = mock_server_with_leaderboard(leaderboard.clone()).await;

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    dry_run,
                )
                .await;
                assert_matches!(result, Ok(BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard: output_leaderboard, changes }) => {
                    assert_eq!(year, TEST_YEAR);
                    assert_eq!(leaderboard_id, TEST_LEADERBOARD_ID);
                    assert_eq!(previous_leaderboard.as_ref(), Some(&base));
                    assert_eq!(output_leaderboard, leaderboard);
                    assert_eq!(changes, expected.changes);
                });

                assert_eq!(storage.len(), 1);
                let (current_leaderboard, current_err) = storage
                    .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                    .await
                    .unwrap();
                assert!(current_err.is_none());
                assert_eq!(current_leaderboard, Some(if dry_run { base } else { leaderboard }));

                if expected.has_changes() && !dry_run {
                    assert!(reporter.called());
                    let (actual_year, actual_leaderboard_id, actual) = &reporter.changes[0];
                    assert_eq!(*actual_year, TEST_YEAR);
                    assert_eq!(*actual_leaderboard_id, TEST_LEADERBOARD_ID);
                    assert_eq!(*actual, expected);
                } else {
                    assert!(!reporter.called())
                }
            }
        }

        mod errors {
            use super::*;

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn leaderboard_get_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_inaccessible_leaderboard)]
                mock_server: MockServer,
                #[values(false, true)] dry_run: bool,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(|_, _| Box::pin(ready(Ok((None, None)))));
                if !dry_run {
                    storage
                        .expect_save_error()
                        .with(
                            eq(TEST_YEAR),
                            eq(TEST_LEADERBOARD_ID),
                            eq(crate::ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)),
                        )
                        .times(1)
                        .returning(move |_, _, _| Box::pin(ready(Ok(()))));
                }

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    dry_run,
                )
                .await;
                assert_matches!(
                    result,
                    Err(crate::Error::Leaderboard(aoc_leaderboard::Error::NoAccess))
                );
                if dry_run {
                    assert!(!reporter.called());
                } else {
                    assert!(reporter.called());
                    assert_eq!(reporter.errors.len(), 1);
                }
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn load_previous_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_leaderboard)]
                mock_server: MockServer,
                #[values(false, true)] dry_run: bool,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| {
                        Box::pin(ready(Err(crate::Error::TestLoadPreviousError)))
                    });
                if !dry_run {
                    storage
                        .expect_save_error()
                        .with(
                            eq(TEST_YEAR),
                            eq(TEST_LEADERBOARD_ID),
                            eq(crate::ErrorKind::Storage(StorageErrorKind::LoadPrevious)),
                        )
                        .times(1)
                        .returning(move |_, _, _| Box::pin(ready(Ok(()))));
                }

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    dry_run,
                )
                .await;
                assert_matches!(result, Err(crate::Error::Storage(StorageError::LoadPrevious(_))));
                if dry_run {
                    assert!(!reporter.called());
                } else {
                    assert!(reporter.called());
                    assert_eq!(reporter.errors.len(), 1);
                    assert_eq!(
                        reporter.errors[0],
                        (
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            "failed to load previous leaderboard data: test".into(),
                        )
                    );
                }
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn report_changes_error(
                config: MemoryConfig,
                #[from(base_leaderboard)] base: Leaderboard,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(leaderboard_with_new_member::default())]
                mock_server: MockServer,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| Box::pin(ready(Ok((Some(base.clone()), None)))));
                storage
                    .expect_save_error()
                    .with(
                        eq(TEST_YEAR),
                        eq(TEST_LEADERBOARD_ID),
                        eq(crate::ErrorKind::Reporter(ReporterErrorKind::ReportChanges)),
                    )
                    .times(1)
                    .returning(move |_, _, _| Box::pin(ready(Ok(()))));

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

                    async fn report_error(
                        &mut self,
                        _year: i32,
                        _leaderboard_id: u64,
                        _error: &crate::Error,
                    ) {
                        self.errors += 1;
                    }
                }

                let mut reporter = MockReporter::default();

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(
                    result,
                    Err(crate::Error::Reporter(ReporterError::ReportChanges(_)))
                );
                assert_eq!(reporter.errors, 1);
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn save_updated_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(leaderboard_with_new_member::default())]
                mock_server: MockServer,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| Box::pin(ready(Ok((Some(base_leaderboard()), None)))));
                storage
                    .expect_save_success()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID), eq(leaderboard_with_new_member()))
                    .times(1)
                    .returning(move |_, _, _| {
                        Box::pin(ready(Err(crate::Error::TestSaveUpdatedError)))
                    });
                storage
                    .expect_save_error()
                    .with(
                        eq(TEST_YEAR),
                        eq(TEST_LEADERBOARD_ID),
                        eq(crate::ErrorKind::Storage(StorageErrorKind::Save)),
                    )
                    .times(1)
                    .returning(move |_, _, _| Box::pin(ready(Ok(()))));

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(result, Err(crate::Error::Storage(StorageError::Save(_))));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(
                    reporter.errors[0],
                    (
                        TEST_YEAR,
                        TEST_LEADERBOARD_ID,
                        "failed to save leaderboard data: test".to_string(),
                    )
                );
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn save_base_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(base_leaderboard::default())]
                mock_server: MockServer,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| Box::pin(ready(Ok((None, None)))));
                storage
                    .expect_save_success()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID), eq(base_leaderboard()))
                    .times(1)
                    .returning(move |_, _, _| {
                        Box::pin(ready(Err(crate::Error::TestSaveBaseError)))
                    });
                storage
                    .expect_save_error()
                    .with(
                        eq(TEST_YEAR),
                        eq(TEST_LEADERBOARD_ID),
                        eq(crate::ErrorKind::Storage(StorageErrorKind::Save)),
                    )
                    .times(1)
                    .returning(move |_, _, _| Box::pin(ready(Ok(()))));

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(result, Err(crate::Error::Storage(StorageError::Save(_))));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(
                    reporter.errors[0],
                    (
                        TEST_YEAR,
                        TEST_LEADERBOARD_ID,
                        "failed to save leaderboard data: test".to_string(),
                    )
                );
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn duplicate_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_inaccessible_leaderboard)]
                mock_server: MockServer,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(|_, _| {
                        Box::pin(ready(Ok((
                            None,
                            Some(crate::ErrorKind::Leaderboard(
                                aoc_leaderboard::ErrorKind::NoAccess,
                            )),
                        ))))
                    });

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(
                    result,
                    Err(crate::Error::Leaderboard(aoc_leaderboard::Error::NoAccess))
                );
                assert!(!reporter.called());
            }
        }
    }
}
