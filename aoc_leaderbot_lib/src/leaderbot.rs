//! Core functionalities of [`aoc_leaderbot`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

pub mod config;
pub mod storage;

use std::collections::HashSet;
use std::future::Future;

use aoc_leaderboard::aoc::Leaderboard;
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};

/// Trait that can be implemented to provide the parameters required by the
/// bot to monitor an [Advent of Code] leaderboard.
///
/// [Advent of Code]: https://adventofcode.com/
pub trait LeaderbotConfig {
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
pub trait LeaderbotStorage {
    /// Type of error used by this storage.
    type Err: std::error::Error + Send;

    /// Loads any previous leaderboard data persisted by a previous bot run.
    ///
    /// If loading was successful but no previous data exists, this method
    /// should return `Ok(None)`.
    fn load_previous(&self) -> impl Future<Output = Result<Option<Leaderboard>, Self::Err>> + Send;

    /// Saves the given leaderboard data to storage so that the next bot run
    /// can fetch it using [`load_previous`](Self::load_previous).
    fn save(
        &mut self,
        leaderboard: &Leaderboard,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send;
}

/// Changes to a leaderboard detected by the bot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderbotChanges {
    /// IDs of new members added to the leaderboard since last run.
    pub new_members: HashSet<u64>,

    /// IDs of members who got new stars since last run.
    pub members_with_new_stars: HashSet<u64>,
}

impl LeaderbotChanges {
    /// Creates a new [`LeaderbotChanges`].
    pub fn new(new_members: HashSet<u64>, members_with_new_stars: HashSet<u64>) -> Self {
        Self { new_members, members_with_new_stars }
    }

    /// Creates a new [`LeaderbotChanges`] if there are new members and/or members
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
pub trait LeaderbotReporter {
    /// Type of error used by this report.
    type Err: std::error::Error + Send;

    /// Report changes to the leaderboard.
    ///
    /// The method receives references to both the previous version of the leaderboard,
    /// the current version of the leaderboard, and the lists of changes detected.
    ///
    /// IDs stored in the [`LeaderbotChanges`] point to [leaderboard members] found
    /// in the current version of the leaderboard.
    ///
    /// [leaderboard members]: Leaderboard::members
    fn report_changes(
        &mut self,
        previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &LeaderbotChanges,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send;

    /// Report an error that occurred while the bot was running.
    ///
    /// This can be useful to report things to the same channel as
    /// the one where we send the leaderboard changes, so that the
    /// bot owner can fix the issue.
    ///
    /// # Notes
    ///
    /// This method doesn't allow returning an error, because it
    /// will only be called while processing another error.
    /// If an error occurs while sending the error report,
    /// it should simply be ignored internally.
    fn report_error<S>(&mut self, error: S) -> impl Future<Output = ()> + Send
    where
        S: Into<String> + Send;
}

/// Runs the bot's core functionality.
///
/// Reads the [`config`], fetches the current leaderboard data, then fetches the previous
/// leaderboard data from [`storage`]. If there was no previous leaderboard (e.g. this is
/// the first run), saves the current leaderboard to storage and exits; otherwise, computes
/// if the leaderboard has new members and/or members who got new stars and calls the
/// [`reporter`] if some diff is found.
///
/// [`config`]: LeaderbotConfig
/// [`storage`]: LeaderbotStorage
/// [`reporter`]: LeaderbotReporter
pub async fn run_bot<C, S, R>(config: C, storage: S, mut reporter: R) -> crate::Result<()>
where
    C: LeaderbotConfig,
    S: LeaderbotStorage,
    R: LeaderbotReporter,
    crate::Error: From<<S as LeaderbotStorage>::Err> + From<<R as LeaderbotReporter>::Err>,
{
    async fn internal_run_bot<C, S, R>(
        config: C,
        mut storage: S,
        reporter: &mut R,
    ) -> crate::Result<()>
    where
        C: LeaderbotConfig,
        S: LeaderbotStorage,
        R: LeaderbotReporter,
        crate::Error: From<<S as LeaderbotStorage>::Err> + From<<R as LeaderbotReporter>::Err>,
    {
        let leaderboard =
            Leaderboard::get(config.year(), config.leaderboard_id(), config.aoc_session()).await?;

        match storage.load_previous().await? {
            Some(previous_leaderboard) => {
                if let Some(changes) = detect_changes(&leaderboard, &previous_leaderboard) {
                    reporter
                        .report_changes(&previous_leaderboard, &leaderboard, &changes)
                        .await?;
                    storage.save(&leaderboard).await?;
                }
            },
            None => storage.save(&leaderboard).await?,
        }

        Ok(())
    }

    match internal_run_bot(config, storage, &mut reporter).await {
        Ok(()) => Ok(()),
        Err(err) => {
            reporter.report_error(err.to_string()).await;
            Err(err)
        },
    }
}

fn detect_changes(
    previous_leaderboard: &Leaderboard,
    leaderboard: &Leaderboard,
) -> Option<LeaderbotChanges> {
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

    LeaderbotChanges::if_needed(new_members, members_with_new_stars)
}
