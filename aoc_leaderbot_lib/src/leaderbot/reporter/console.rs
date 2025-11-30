//! Bot reporter that outputs to the console.

mod detail;

use std::cmp::Ordering;
use itertools::Itertools;
use aoc_leaderboard::aoc::{Leaderboard, LeaderboardMember};
use crate::leaderbot::{Changes, Reporter};
use crate::leaderbot::reporter::console::detail::{ConsoleReporterStringExt, STARS_HEADER};

/// Bot reporter that outputs to the console.
///
/// [Changes] and [first runs] are reported to `stdout`, while [errors] are reported to `stderr`.
///
/// [Changes]: Reporter::report_changes
/// [first runs]: Reporter::report_first_run
/// [errors]: Reporter::report_error
#[derive(Debug, Default, Clone)]
pub struct ConsoleReporter;

impl ConsoleReporter {
    fn message_text(
        &self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
        changes: Option<&Changes>,
    ) -> String {
        let mut member_rows = leaderboard
            .members
            .values()
            .sorted_by(|lhs, rhs| self.compare_members(lhs, rhs))
            .map(|member| self.member_row_text(member, changes));

        format!(
            "{}Leaderboard {leaderboard_id} (year {year})\n{}",
            STARS_HEADER.right_pad(12, '\u{2007}'),
            member_rows.join("\n")
        )
    }

    fn compare_members(&self, lhs: &LeaderboardMember, rhs: &LeaderboardMember) -> Ordering {
        // Comparing by `last_star_ts` will prioritize those that got their latest star first.
        // I think AoC does this, but I'm not 100% sure.
        lhs
            .stars
            .cmp(&rhs.stars)
            .then_with(|| lhs.local_score.cmp(&rhs.local_score))
            .then_with(|| lhs.last_star_ts.cmp(&rhs.last_star_ts))
            .then_with(|| lhs.id.cmp(&rhs.id))
    }

    fn member_row_text(&self, member: &LeaderboardMember, changes: Option<&Changes>) -> String {
        let row_text = format!(
            "{}{}",
            member.stars.to_string().right_pad(12, '\u{2007}'),
            member
                .name
                .clone()
                .unwrap_or_else(|| format!("(anonymous user #{})", member.id)),
        );
        self.add_member_row_emoji(row_text, member, changes)
    }

    // noinspection DuplicatedCode
    fn add_member_row_emoji(
        &self,
        row_text: String,
        member: &LeaderboardMember,
        changes: Option<&Changes>,
    ) -> String {
        if changes.is_some_and(|c| c.new_members.contains(&member.id)) {
            format!("*{row_text} 👋*")
        } else if changes.is_some_and(|c| c.members_with_new_stars.contains(&member.id)) {
            format!("*{row_text} 🎉*")
        } else {
            row_text
        }
    }
}

impl Reporter for ConsoleReporter {
    type Err = crate::Error;

    #[cfg_attr(
        not(coverage),
        tracing::instrument(
            skip(self, _view_key, _previous_leaderboard, leaderboard, changes),
            err
        )
    )]
    async fn report_changes(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        _view_key: Option<&str>,
        _previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &Changes,
    ) -> Result<(), Self::Err> {
        let message = self.message_text(year, leaderboard_id, leaderboard, Some(changes));
        println!("{message}");

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(skip(self, _view_key, leaderboard), err))]
    async fn report_first_run(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        _view_key: Option<&str>,
        leaderboard: &Leaderboard,
    ) -> Result<(), Self::Err> {
        let message = self.message_text(year, leaderboard_id, leaderboard, None);
        println!("{message}");

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(skip(self, _view_key, error)))]
    async fn report_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        _view_key: Option<&str>,
        error: &crate::Error,
    ) {
        eprintln!("An error occurred for leaderboard {leaderboard_id} and year {year}: {error}");
    }
}
