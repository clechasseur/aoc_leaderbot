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
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{ReporterError, StorageError};

/// Trait that must be implemented to provide the parameters required by the
/// bot to monitor an [Advent of Code] leaderboard.
///
/// [Advent of Code]: https://adventofcode.com/
pub trait Config {
    /// Year for which we want to monitor the leaderboard.
    ///
    /// Defaults to the current year.
    #[instrument(skip(self), level = "trace", ret)]
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
    /// Returns a [`Changes`] with the given new/updated members.
    #[instrument(level = "trace")]
    pub fn new(new_members: HashSet<u64>, members_with_new_stars: HashSet<u64>) -> Self {
        Self { new_members, members_with_new_stars }
    }

    /// Returns a [`Changes`] if there are new members and/or members
    /// with new stars, otherwise returns `None`.
    #[instrument(level = "trace", ret)]
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
    #[instrument(skip(self))]
    fn report_error<S>(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error: S,
    ) -> impl Future<Output = ()> + Send
    where
        S: Into<String> + Debug + Send,
    {
        let error = error.into();
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

impl BotOutput {
    /// Creates a new [`BotOutput`] with current leaderboard data.
    ///
    /// The [`previous_leaderboard`](Self::previous_leaderboard) and
    /// [`changes`](Self::changes) field will both be set to `None`.
    #[instrument(level = "trace", ret)]
    pub fn new(year: i32, leaderboard_id: u64, leaderboard: Leaderboard) -> Self {
        Self { year, leaderboard_id, previous_leaderboard: None, leaderboard, changes: None }
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
/// If the `dry_run` parameter is set to `true`, then the bot will fetch data and compute
/// changes but will not persist or report them.
///
/// [`config`]: Config
/// [`storage`]: Storage
/// [`reporter`]: Reporter
#[cfg_attr(coverage_nightly, coverage(off))]
#[instrument(skip(config, storage, reporter), ret, err)]
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
#[instrument(skip(config, storage, reporter), level = "debug", ret, err)]
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
    async fn internal_run_bot<B, S, R>(
        advent_of_code_base: Option<B>,
        year: i32,
        leaderboard_id: u64,
        aoc_session: &str,
        storage: &mut S,
        reporter: &mut R,
        dry_run: bool,
    ) -> crate::Result<BotOutput>
    where
        B: AsRef<str> + Debug,
        S: Storage,
        <S as Storage>::Err: Error + Sync + 'static,
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

        let reporter: &mut R = reporter;

        let leaderboard =
            get_leaderboard(advent_of_code_base, year, leaderboard_id, aoc_session).await?;
        let mut output = BotOutput::new(year, leaderboard_id, leaderboard.clone());

        let load_result = storage
            .load_previous(year, leaderboard_id)
            .await
            .map_err(|err| StorageError::LoadPrevious(anyhow!(err)))?;
        match load_result {
            Some(previous_leaderboard) => {
                output.previous_leaderboard = Some(previous_leaderboard.clone());

                if let Some(changes) = detect_changes(&previous_leaderboard, &leaderboard) {
                    output.changes = Some(changes.clone());

                    if !dry_run {
                        reporter
                            .report_changes(
                                year,
                                leaderboard_id,
                                &previous_leaderboard,
                                &leaderboard,
                                &changes,
                            )
                            .await
                            .map_err(|err| ReporterError::ReportChanges(anyhow!(err)))?;
                        storage
                            .save(year, leaderboard_id, &leaderboard)
                            .await
                            .map_err(|err| StorageError::Save(anyhow!(err)))?;
                    }
                }
            },
            None if dry_run => (),
            None => storage
                .save(year, leaderboard_id, &leaderboard)
                .await
                .map_err(|err| StorageError::Save(anyhow!(err)))?,
        }

        Ok(output)
    }

    let (year, leaderboard_id, aoc_session) =
        (config.year(), config.leaderboard_id(), config.aoc_session());

    match internal_run_bot(
        advent_of_code_base,
        year,
        leaderboard_id,
        &aoc_session,
        storage,
        reporter,
        dry_run,
    )
    .await
    {
        Ok(output) => Ok(output),
        Err(err) => {
            reporter
                .report_error(year, leaderboard_id, err.to_string())
                .await;
            Err(err)
        },
    }
}

#[instrument(ret)]
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

    mod reporter {
        use aoc_leaderbot_test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};

        use super::*;

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

        #[test_log::test(tokio::test)]
        async fn default_impl_works() {
            let mut reporter = TestReporter;

            let leaderboard = get_sample_leaderboard();
            let changes = Changes::new([42, 23].into(), [11, 7].into());

            reporter
                .report_changes(YEAR, LEADERBOARD_ID, &leaderboard, &leaderboard, &changes)
                .await
                .unwrap();

            reporter
                .report_error(YEAR, LEADERBOARD_ID, "Something went wrong")
                .await;
        }
    }

    mod run_bot {
        use std::collections::HashMap;
        use std::future::ready;

        use aoc_leaderboard::aoc::{CompletionDayLevel, LeaderboardMember, PuzzleCompletionInfo};
        use aoc_leaderbot_test_helpers::{
            get_mock_server_with_leaderboard, AOC_SESSION, LEADERBOARD_ID, YEAR,
        };
        use assert_matches::assert_matches;
        use mockall::predicate::eq;

        use super::*;
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

            async fn report_error<S>(&mut self, year: i32, leaderboard_id: u64, error: S)
            where
                S: Into<String> + Debug + Send,
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

        // noinspection DuplicatedCode
        mod without_previous {
            use super::*;

            #[test_log::test(tokio::test)]
            async fn stores_current() {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let mock_server = get_mock_server_with_leaderboard(base_leaderboard()).await;

                let expected = base_leaderboard();
                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(result, Ok(BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard, changes }) => {
                    assert_eq!(year, YEAR);
                    assert_eq!(leaderboard_id, LEADERBOARD_ID);
                    assert!(previous_leaderboard.is_none());
                    assert_eq!(leaderboard, expected);
                    assert!(changes.is_none());
                });
                assert_eq!(storage.len(), 1);
                assert!(!reporter.called());

                let actual = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
                assert_eq!(actual, Some(expected));
            }

            mod dry_run {
                use super::*;

                #[test_log::test(tokio::test)]
                async fn does_not_store_current() {
                    let config = config();
                    let mut storage = storage();
                    let mut reporter = spy_reporter();

                    let mock_server = get_mock_server_with_leaderboard(base_leaderboard()).await;

                    let expected = base_leaderboard();
                    let result = run_bot_from(
                        Some(mock_server.uri()),
                        &config,
                        &mut storage,
                        &mut reporter,
                        true,
                    )
                    .await;
                    assert_matches!(result, Ok(BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard, changes }) => {
                        assert_eq!(year, YEAR);
                        assert_eq!(leaderboard_id, LEADERBOARD_ID);
                        assert!(previous_leaderboard.is_none());
                        assert_eq!(leaderboard, expected);
                        assert!(changes.is_none());
                    });
                    assert!(storage.is_empty());
                    assert!(!reporter.called());

                    let actual = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
                    assert!(actual.is_none());
                }
            }
        }

        mod with_previous {
            use itertools::iproduct;

            use super::*;

            async fn test_previous<N, W>(
                leaderboard: Leaderboard,
                dry_run: bool,
                expected_new_members: N,
                expected_members_with_new_stars: W,
            ) where
                N: IntoIterator<Item = u64>,
                W: IntoIterator<Item = u64>,
            {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let base = base_leaderboard();

                storage.save(YEAR, LEADERBOARD_ID, &base).await.unwrap();

                let mock_server = get_mock_server_with_leaderboard(leaderboard.clone()).await;

                let expected = SpiedChanges {
                    previous_leaderboard: base.clone(),
                    leaderboard: leaderboard.clone(),
                    changes: Changes::if_needed(
                        expected_new_members.into_iter().collect(),
                        expected_members_with_new_stars.into_iter().collect(),
                    ),
                };

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    dry_run,
                )
                .await;
                assert_matches!(result, Ok(BotOutput { year, leaderboard_id, previous_leaderboard, leaderboard: output_leaderboard, changes }) => {
                    assert_eq!(year, YEAR);
                    assert_eq!(leaderboard_id, LEADERBOARD_ID);
                    assert_eq!(previous_leaderboard.as_ref(), Some(&base));
                    assert_eq!(output_leaderboard, leaderboard);
                    assert_eq!(changes, expected.changes);
                });

                assert_eq!(storage.len(), 1);
                let current = storage.load_previous(YEAR, LEADERBOARD_ID).await.unwrap();
                assert_eq!(current, Some(if dry_run { base } else { leaderboard }));

                if expected.has_changes() && !dry_run {
                    assert!(reporter.called());
                    let (actual_year, actual_leaderboard_id, actual) = &reporter.changes[0];
                    assert_eq!(*actual_year, YEAR);
                    assert_eq!(*actual_leaderboard_id, LEADERBOARD_ID);
                    assert_eq!(*actual, expected);
                } else {
                    assert!(!reporter.called())
                }
            }

            fn test_permutations() -> impl Iterator<Item = ((Leaderboard, Vec<u64>, Vec<u64>), bool)>
            {
                iproduct!(
                    // leaderboard, expected_new_members, expected_members_with_new_stars
                    [
                        (base_leaderboard(), vec![], vec![]),
                        (leaderboard_with_new_member(), vec![MEMBER_2], vec![]),
                        (leaderboard_with_member_with_new_stars(), vec![], vec![MEMBER_1]),
                        (leaderboard_with_both_updates(), vec![MEMBER_2], vec![MEMBER_1]),
                    ],
                    // dry_run
                    [false, true]
                )
            }

            #[test_log::test(tokio::test)]
            async fn all() {
                let permutations = test_permutations();

                for (
                    (leaderboard, expected_new_members, expected_members_with_new_stars),
                    dry_run,
                ) in permutations
                {
                    test_previous(
                        leaderboard,
                        dry_run,
                        expected_new_members,
                        expected_members_with_new_stars,
                    )
                    .await;
                }
            }
        }

        mod errors {
            use aoc_leaderbot_test_helpers::get_mock_server_with_inaccessible_leaderboard;

            use super::*;

            #[test_log::test(tokio::test)]
            async fn leaderboard_get_error() {
                let config = config();
                let mut storage = storage();
                let mut reporter = spy_reporter();

                let mock_server = get_mock_server_with_inaccessible_leaderboard().await;

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
                assert!(storage.is_empty());
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
            }

            #[test_log::test(tokio::test)]
            async fn load_previous_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let mock_server = get_mock_server_with_leaderboard(base_leaderboard()).await;

                storage
                    .expect_load_previous()
                    .with(eq(YEAR), eq(LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| {
                        Box::pin(ready(Err(crate::Error::TestLoadPreviousError)))
                    });

                let result = run_bot_from(
                    Some(mock_server.uri()),
                    &config,
                    &mut storage,
                    &mut reporter,
                    false,
                )
                .await;
                assert_matches!(result, Err(crate::Error::Storage(StorageError::LoadPrevious(_))));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(
                    reporter.errors[0],
                    (
                        YEAR,
                        LEADERBOARD_ID,
                        "failed to load previous leaderboard data: test".to_string()
                    )
                );
            }

            #[test_log::test(tokio::test)]
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
                        S: Into<String> + Debug + Send,
                    {
                        self.errors += 1;
                    }
                }

                let config = config();
                let mut storage = storage();
                let mut reporter = MockReporter::default();

                let base = base_leaderboard();
                storage.save(YEAR, LEADERBOARD_ID, &base).await.unwrap();

                let mock_server =
                    get_mock_server_with_leaderboard(leaderboard_with_new_member()).await;

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

            #[test_log::test(tokio::test)]
            async fn save_updated_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let mock_server =
                    get_mock_server_with_leaderboard(leaderboard_with_new_member()).await;

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
                    (YEAR, LEADERBOARD_ID, "failed to save leaderboard data: test".to_string())
                );
            }

            #[test_log::test(tokio::test)]
            async fn save_base_error() {
                let config = config();
                let mut storage = MockStorage::new();
                let mut reporter = spy_reporter();

                let mock_server = get_mock_server_with_leaderboard(base_leaderboard()).await;

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
                    (YEAR, LEADERBOARD_ID, "failed to save leaderboard data: test".to_string())
                );
            }
        }
    }
}
