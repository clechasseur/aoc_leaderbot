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
use crate::ErrorKind;

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

/// Data that can be stored by the bot in a [`Storage`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotData {
    /// Optional leaderboard data.
    ///
    /// Will be used to store the previous leaderboard data when the bot executes.
    pub leaderboard: Option<Leaderboard>,

    /// Optional error.
    ///
    /// Will be used to store information about any error that is encountered
    /// during bot execution, so that we know not to report the same error
    /// more than once.
    pub error: Option<BotErrorData>,
}

impl BotData {
    /// Creates a bot data instance with a leaderboard instance and no error.
    pub fn for_leaderboard(leaderboard: Leaderboard) -> Self {
        Self { leaderboard: Some(leaderboard), error: None }
    }
}

/// Data associated with an error that occurred while the bot executes.
///
/// Will be stored in [`BotData`] when such an error occurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotErrorData {
    /// Kind of error encountered.
    pub kind: ErrorKind,

    /// Optional error message.
    pub message: Option<String>,
}

/// Trait used to augment types to split a `BotData`.
pub trait BotDataSplitExt {
    /// Type of first tuple element returned by [`split`](Self::split).
    /// Will be either [`Leaderboard`] or a reference to it.
    type LeaderboardType;

    /// Type of second tuple element returned by [`split`](Self::split).
    /// Will be either [`BotErrorData`] or a reference to it.
    type BotErrorDataType;

    /// Splits `self`, which contains a [`BotData`], into a tuple of its inner
    /// [`Leaderboard`] and [`BotErrorData`] (or references to them).
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>);
}

impl BotDataSplitExt for BotData {
    type LeaderboardType = Leaderboard;
    type BotErrorDataType = BotErrorData;

    /// Splits this [`BotData`] into a tuple of [`Option`]s of its [`Leaderboard`]
    /// and [`BotErrorData`].
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        (self.leaderboard, self.error)
    }
}

impl<'a> BotDataSplitExt for &'a BotData {
    type LeaderboardType = &'a Leaderboard;
    type BotErrorDataType = &'a BotErrorData;

    /// Splits this [`BotData`] reference into a tuple of [`Option`]s of references
    /// to its [`Leaderboard`] and [`BotErrorData`].
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        (self.leaderboard.as_ref(), self.error.as_ref())
    }
}

impl<S> BotDataSplitExt for Option<S>
where
    S: BotDataSplitExt,
{
    type LeaderboardType = S::LeaderboardType;
    type BotErrorDataType = S::BotErrorDataType;

    /// If this [`Option`] is [`Some(value)`], splits `value` into a tuple of corresponding
    /// [`Leaderboard`] and [`BotErrorData`], otherwise returns `(None, None)`.
    // noinspection DuplicatedCode
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        self.map(|splittable| splittable.split()).unwrap_or((None, None))
    }
}

impl<'a, S> BotDataSplitExt for &'a Option<S>
where
    &'a S: BotDataSplitExt,
{
    type LeaderboardType = <&'a S as BotDataSplitExt>::LeaderboardType;
    type BotErrorDataType = <&'a S as BotDataSplitExt>::BotErrorDataType;

    /// If this [`Option`] reference is [`Some(value)`], splits `value` into a tuple of
    /// corresponding [`Leaderboard`] and [`BotErrorData`] references, otherwise returns
    /// `(None, None)`.
    // noinspection DuplicatedCode
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        self.as_ref().map(|splittable| splittable.split()).unwrap_or((None, None))
    }
}

impl<S, E> BotDataSplitExt for Result<S, E>
where
    S: BotDataSplitExt,
{
    type LeaderboardType = S::LeaderboardType;
    type BotErrorDataType = S::BotErrorDataType;

    /// If this [`Result`] is [`Ok(value)`], splits `value` into a tuple of corresponding
    /// [`Leaderboard`] and [`BotErrorData`], otherwise returns `(None, None)`, dropping
    /// the error.
    // noinspection DuplicatedCode
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        self.map(|splittable| splittable.split()).unwrap_or((None, None))
    }
}

impl<'a, S, E> BotDataSplitExt for &'a Result<S, E>
where
    &'a S: BotDataSplitExt,
{
    type LeaderboardType = <&'a S as BotDataSplitExt>::LeaderboardType;
    type BotErrorDataType = <&'a S as BotDataSplitExt>::BotErrorDataType;

    /// If this [`Result`] reference is [`Ok(value)`], splits `value` into a tuple of
    /// corresponding [`Leaderboard`] and [`BotErrorData`] references , otherwise returns
    /// `(None, None)`.
    // noinspection DuplicatedCode
    fn split(self) -> (Option<Self::LeaderboardType>, Option<Self::BotErrorDataType>) {
        self.as_ref().map(|splittable| splittable.split()).unwrap_or((None, None))
    }
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
    ) -> impl Future<Output = Result<Option<BotData>, Self::Err>> + Send;

    /// Saves the given leaderboard data to storage so that the next bot run
    /// can fetch it using [`load_previous`](Self::load_previous).
    fn save(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        bot_data: &BotData,
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
    ///
    /// If the bot gets the same error twice, it will include
    /// information about the previous error.
    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self)))]
    fn report_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error: &crate::Error,
        previous_error: Option<&BotErrorData>,
    ) -> impl Future<Output = ()> + Send {
        eprintln!("Error while looking for changes to leaderboard {leaderboard_id} for year {year}: {error}");
        if let Some(previous_error) = previous_error {
            eprintln!("This is the same error that happened during last run; previous error info: {previous_error:?}");
        }
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
    #[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace", ret))]
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
    async fn internal_run_bot<B, S, R>(
        advent_of_code_base: Option<B>,
        year: i32,
        leaderboard_id: u64,
        aoc_session: &str,
        storage: &mut S,
        reporter: &mut R,
        dry_run: bool,
    ) -> crate::Result<BotOutput, (crate::Error, Option<BotData>)>
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

        let leaderboard_result =
            get_leaderboard(advent_of_code_base, year, leaderboard_id, aoc_session).await;
        let load_previous_result = storage
            .load_previous(year, leaderboard_id)
            .await
            .map_err(|err| StorageError::LoadPrevious(anyhow!(err)));
        let (leaderboard, previous_data) = match (leaderboard_result, load_previous_result) {
            (Err(get_err), load_res) => {
                return Err((get_err, load_res.ok().flatten()));
            },
            (Ok(leaderboard), Err(load_err)) => {
                return Err((load_err.into(), Some(BotData::for_leaderboard(leaderboard))))
            },
            (Ok(leaderboard), Ok(bot_data)) => (leaderboard, bot_data),
        };

        let mut output = BotOutput::new(year, leaderboard_id, leaderboard.clone());
        let (previous_leaderboard, previous_error) = previous_data.split();
        let leaderboard_and_prev_error = match (previous_leaderboard, previous_error) {
            (Some(previous_leaderboard), prev_error) => {
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
                            .map_err(|err| {
                                let bot_data = BotData {
                                    leaderboard: Some(leaderboard.clone()),
                                    error: prev_error.clone(),
                                };
                                let reporter_err = ReporterError::ReportChanges(anyhow!(err));
                                (reporter_err.into(), Some(bot_data))
                            })?;
                    }
                }

                Some((leaderboard, prev_error))
            },
            (None, _) if dry_run => None,
            (None, prev_error) => Some((leaderboard, prev_error)),
        };
        if let Some((leaderboard, prev_error)) = leaderboard_and_prev_error {
            let bot_data = BotData::for_leaderboard(leaderboard);

            storage
                .save(year, leaderboard_id, &bot_data)
                .await
                .map_err(|err| {
                    let bot_data = BotData {
                        leaderboard: bot_data.leaderboard,
                        error: prev_error.clone(),
                    };
                    let storage_err = StorageError::Save(anyhow!(err));
                    (storage_err.into(), Some(bot_data))
                })?;
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
        Err((err, bot_data)) => {
            let (leaderboard, prev_error) = bot_data.as_ref().split();

            reporter.report_error(
                year,
                leaderboard_id,
                &err,
                prev_error,
            ).await;

            if !dry_run {
                let new_bot_data = BotData {
                    leaderboard: leaderboard.cloned(),
                    error: Some(BotErrorData {
                        kind: err.discriminant(),
                        message: Some(err.to_string()),
                    }),
                };
                // Don't try to save previous error data if we got the same result as before.
                if !bot_data.as_ref().is_some_and(|data| *data == new_bot_data) {
                    if let Err(storage_err) = storage.save(year, leaderboard_id, &new_bot_data).await {
                        // An error occurred while doing the bot run, and an error also occurred
                        // while trying to persist information about the last error. ¯\_(ツ)_/¯
                        let storage_err = crate::Error::Storage(StorageError::Save(anyhow!(storage_err)));
                        reporter.report_error(
                            year,
                            leaderboard_id,
                            &storage_err,
                            None,
                        ).await;
                    }
                }
            }

            Err(err)
        },
    }
}

#[cfg_attr(not(coverage_nightly), tracing::instrument(ret))]
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
                    Some(&BotErrorData {
                        kind: ErrorKind::TestErrorWithMessage,
                        message: Some("broken pipe".into()),
                    }),
                )
                .await;
        }
    }

    mod run_bot {
        use std::collections::HashMap;
        use std::future::ready;

        use aoc_leaderboard::aoc::{CompletionDayLevel, LeaderboardMember, PuzzleCompletionInfo};
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
            pub errors: Vec<(i32, u64, String, String)>,
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

            async fn report_error(
                &mut self,
                year: i32,
                leaderboard_id: u64,
                error: &crate::Error,
                prev_error: Option<&BotErrorData>,
            ) {
                self.errors.push((
                    year,
                    leaderboard_id,
                    error.to_string(),
                    prev_error.and_then(|prev_error| prev_error.message.clone()).unwrap_or_default(),
                ));
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

        // noinspection DuplicatedCode
        mod without_previous {
            use aoc_leaderboard::wiremock::MockServer;

            use super::*;

            #[rstest]
            #[case::stores_current(false)]
            #[case::dry_run::does_not_store_current(true)]
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

                let actual = storage
                    .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                    .await
                    .unwrap();
                assert_eq!(actual, if dry_run { None } else { Some(BotData::for_leaderboard(expected)) });
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
                let bot_data = BotData::for_leaderboard(base);
                storage
                    .save(TEST_YEAR, TEST_LEADERBOARD_ID, &bot_data)
                    .await
                    .unwrap();

                let base = bot_data.leaderboard.unwrap();
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
                let current = storage
                    .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                    .await
                    .unwrap();
                assert_eq!(current, Some(BotData::for_leaderboard(
                    if dry_run { base } else { leaderboard })
                ));

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
            use aoc_leaderboard::wiremock::MockServer;

            use super::*;

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn leaderboard_get_error(
                config: MemoryConfig,
                mut storage: MemoryStorage,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_inaccessible_leaderboard)]
                mock_server: MockServer,
            ) {
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
                assert_eq!(storage.len(), 1);
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn load_previous_error(
                config: MemoryConfig,
                mut reporter: SpyReporter,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(base_leaderboard::default())]
                mock_server: MockServer,
                #[from(base_leaderboard)]
                expected_leaderboard: Leaderboard,
            ) {
                let mut storage = MockStorage::new();
                storage
                    .expect_load_previous()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID))
                    .times(1)
                    .returning(move |_, _| {
                        Box::pin(ready(Err(crate::Error::TestLoadPreviousError)))
                    });
                storage
                    .expect_save()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID), eq(BotData {
                        leaderboard: Some(expected_leaderboard),
                        error: Some(BotErrorData {
                            kind: ErrorKind::TestLoadPreviousError,
                            message: Some("failed to load previous leaderboard data: test".into()),
                        }),
                    }))
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
                assert_matches!(result, Err(crate::Error::Storage(StorageError::LoadPrevious(_))));
                assert!(reporter.called());
                assert_eq!(reporter.errors.len(), 1);
                assert_eq!(
                    reporter.errors[0],
                    (
                        TEST_YEAR,
                        TEST_LEADERBOARD_ID,
                        "failed to load previous leaderboard data: test".into(),
                        "".into(),
                    )
                );
            }

            #[rstest]
            #[awt]
            #[test_log::test(tokio::test)]
            async fn report_changes_error(
                config: MemoryConfig,
                mut storage: MemoryStorage,
                #[from(base_leaderboard)] base: Leaderboard,
                #[future]
                #[from(mock_server_with_leaderboard)]
                #[with(leaderboard_with_new_member::default())]
                mock_server: MockServer,
            ) {
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
                        _prev_error: Option<&BotErrorData>,
                    ) {
                        self.errors += 1;
                    }
                }

                let mut reporter = MockReporter::default();

                storage
                    .save(TEST_YEAR, TEST_LEADERBOARD_ID, &BotData::for_leaderboard(base))
                    .await
                    .unwrap();

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
                    .returning(move |_, _| Box::pin(ready(Ok(Some(
                        BotData::for_leaderboard(base_leaderboard())
                    )))));
                storage
                    .expect_save()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID), eq(
                        BotData::for_leaderboard(leaderboard_with_new_member())
                    ))
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
                    (
                        TEST_YEAR,
                        TEST_LEADERBOARD_ID,
                        "failed to save leaderboard data: test".to_string(),
                        "".into(),
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
                    .returning(move |_, _| Box::pin(ready(Ok(None))));
                storage
                    .expect_save()
                    .with(eq(TEST_YEAR), eq(TEST_LEADERBOARD_ID), eq(
                        BotData::for_leaderboard(base_leaderboard())
                    ))
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
                    (
                        TEST_YEAR,
                        TEST_LEADERBOARD_ID,
                        "failed to save leaderboard data: test".to_string(),
                        "".into(),
                    )
                );
            }

            // TODO add tests with previous error
        }
    }
}
