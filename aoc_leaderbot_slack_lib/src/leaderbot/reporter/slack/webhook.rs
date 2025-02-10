//! Implementations of [`leaderbot::Reporter`] using [Slack webhooks].
//!
//! [`leaderbot::Reporter`]: Reporter
//! [Slack webhooks]: https://api.slack.com/messaging/webhooks

mod detail;

use std::cmp::Ordering;
use std::env;

use aoc_leaderboard::aoc::{Leaderboard, LeaderboardMember};
use aoc_leaderbot_lib::leaderbot::{Changes, Reporter};
use derive_builder::Builder;
use itertools::Itertools;
use strum::{Display, EnumProperty, EnumString};
use tracing::{error, instrument, trace};

use crate::error::WebhookError;
use crate::leaderbot::reporter::slack::webhook::detail::SlackWebhookReporterStringExt;
use crate::leaderbot::reporter::slack::USER_AGENT;
use crate::slack::webhook::WebhookMessage;

/// Environment variable from which the Slack webhook URL will be
/// fetched if not specified.
pub const WEBHOOK_URL_ENV_VAR: &str = "SLACK_WEBHOOK_URL";

/// Environment variable from which the Slack channel will be fetched
/// if not specified.
pub const CHANNEL_ENV_VAR: &str = "SLACK_CHANNEL";

/// Environment variable from which the leaderboard members
/// sort order will be fetched if not specified.
pub const SORT_ORDER_ENV_VAR: &str = "SLACK_LEADERBOARD_SORT_ORDER";

/// Possible sort order of members when reporting leaderboard changes.
///
/// The default sort order is [`Stars`](Self::Stars).
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumProperty,
    EnumString,
)]
pub enum LeaderboardSortOrder {
    /// Sort leaderboard members by number of stars, descending.
    #[default]
    #[strum(serialize = "stars", props(header = "Stars ⭐"))]
    Stars,

    /// Sort leaderboard members by score, descending.
    #[strum(serialize = "local_score", props(header = "Score #"))]
    Score,
}

impl LeaderboardSortOrder {
    /// Compares two [`LeaderboardMember`]s using this sort order.
    ///
    /// If the members are [`Equal`](Ordering::Equal) according to the chosen
    /// sort value (ex: stars), they will then be compared using the other
    /// possible sort value (ex: score), then by [`last_star_ts`](LeaderboardMember::last_star_ts)
    /// then finally by [`id`](LeaderboardMember::id) for a stable sort.
    pub fn cmp_members(&self, lhs: &LeaderboardMember, rhs: &LeaderboardMember) -> Ordering {
        let ordering = match *self {
            Self::Stars => rhs
                .stars
                .cmp(&lhs.stars)
                .then_with(|| rhs.local_score.cmp(&lhs.local_score)),
            Self::Score => rhs
                .local_score
                .cmp(&lhs.local_score)
                .then_with(|| rhs.stars.cmp(&lhs.stars)),
        };

        // Comparing by `last_star_ts` will prioritize those that got their latest star first.
        // I think AoC does this, but I'm not 100% sure.
        ordering
            .then_with(|| lhs.last_star_ts.cmp(&rhs.last_star_ts))
            .then_with(|| lhs.id.cmp(&rhs.id))
    }

    /// Returns a string representation of the value that would be used
    /// to sort the given [`LeaderboardMember`] according to this sort order.
    pub fn member_value_text(&self, member: &LeaderboardMember) -> String {
        let value_text = match *self {
            Self::Stars => member.stars.to_string(),
            Self::Score => member.local_score.to_string(),
        };

        value_text.right_pad(12, '\u{2007}')
    }

    /// Returns the header text to display in a message when this sort order is used.
    pub fn header_text(&self) -> String {
        self.get_str("header").unwrap().right_pad(12, '\u{2007}')
    }
}

/// An [`aoc_leaderbot`] [`Reporter`] that sends leaderboard updates
/// to a Slack channel via a [Slack webhook] URL.
///
/// [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
/// [Slack webhook]: https://api.slack.com/messaging/webhooks
#[derive(Debug, Clone, Builder)]
#[builder(derive(Debug), build_fn(name = "build_internal", private))]
pub struct SlackWebhookReporter {
    /// Slack webhook URL used to send leaderboard updates.
    #[builder(setter(into), default = "Self::default_webhook_url()?")]
    pub webhook_url: String,

    /// Slack channel to post leaderboard updates to.
    #[builder(setter(into), default = "Self::default_channel()?")]
    pub channel: String,

    /// Username used when posting messages to Slack.
    #[builder(
        setter(into),
        default = "crate::leaderbot::reporter::slack::DEFAULT_USERNAME.into()"
    )]
    pub username: String,

    /// URL of an icon to use to post messages to Slack.
    #[builder(
        setter(into),
        default = "crate::leaderbot::reporter::slack::DEFAULT_ICON_URL.into()"
    )]
    pub icon_url: String,

    /// Sort order of leaderboard members. Used when [reporting changes](Reporter::report_changes).
    ///
    /// Defaults to [`Stars`](LeaderboardSortOrder::Stars).
    #[builder(default = "Self::default_sort_order()?")]
    pub sort_order: LeaderboardSortOrder,

    #[builder(private, default = "Self::default_http_client()?")]
    http_client: reqwest::Client,
}

impl SlackWebhookReporter {
    /// Returns a [builder](SlackWebhookReporterBuilder) that can be used
    /// to customize a Slack webhook reporter.
    pub fn builder() -> SlackWebhookReporterBuilder {
        SlackWebhookReporterBuilder::default()
    }

    fn message_text(&self, leaderboard: &Leaderboard, changes: &Changes) -> String {
        let mut member_rows = leaderboard
            .members
            .values()
            .sorted_by(|lhs, rhs| self.sort_order.cmp_members(lhs, rhs))
            .map(|member| self.member_row_text(member, changes));

        format!("{}\n{}", self.header_row_text(leaderboard), member_rows.join("\n"))
    }

    fn member_row_text(&self, member: &LeaderboardMember, changes: &Changes) -> String {
        let row_text = format!(
            "{}{}",
            self.sort_order.member_value_text(member),
            member
                .name
                .clone()
                .unwrap_or_else(|| format!("(anonymous user #{})", member.id)),
        );
        self.add_member_row_emoji(row_text, member, changes)
    }

    fn add_member_row_emoji(
        &self,
        row_text: String,
        member: &LeaderboardMember,
        changes: &Changes,
    ) -> String {
        if changes.new_members.contains(&member.id) {
            format!("*{row_text} 👋*")
        } else if changes.members_with_new_stars.contains(&member.id) {
            format!("*{row_text} 🎉*")
        } else {
            row_text
        }
    }

    fn header_row_text(&self, leaderboard: &Leaderboard) -> String {
        format!("*{}{}*", self.sort_order.header_text(), self.leaderboard_link(leaderboard))
    }

    fn leaderboard_link(&self, leaderboard: &Leaderboard) -> String {
        format!(
            "<https://adventofcode.com/{}/leaderboard/private/view?order={}|*Leaderboard*>",
            leaderboard.year, self.sort_order
        )
    }
}

impl SlackWebhookReporterBuilder {
    /// Builds the [`SlackWebhookReporter`].
    pub fn build(&self) -> crate::Result<SlackWebhookReporter> {
        self.build_internal().map_err(Into::into)
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(
        &self,
    ) -> Result<SlackWebhookReporter, SlackWebhookReporterBuilderError> {
        self.build_internal()
    }

    fn default_webhook_url() -> Result<String, String> {
        Self::env_var(WEBHOOK_URL_ENV_VAR, "webhook_url")
    }

    fn default_channel() -> Result<String, String> {
        Self::env_var(CHANNEL_ENV_VAR, "channel")
    }

    fn default_sort_order() -> Result<LeaderboardSortOrder, String> {
        match env::var(SORT_ORDER_ENV_VAR) {
            Ok(sort_order) => sort_order.parse().map_err(|_| {
                format!(
                    "invalid sort_order specified in environment variable {SORT_ORDER_ENV_VAR}: {}",
                    sort_order
                )
            }),
            Err(env::VarError::NotPresent) => Ok(LeaderboardSortOrder::default()),
            Err(env::VarError::NotUnicode(val)) => Err(format!(
                "invalid unicode found in environment variable {SORT_ORDER_ENV_VAR}: {}",
                val.to_string_lossy(),
            )),
        }
    }

    fn default_http_client() -> Result<reqwest::Client, String> {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|err| format!("error building HTTP client: {err}"))
    }

    fn env_var(var_name: &str, field_name: &str) -> Result<String, String> {
        env::var(var_name).map_err(|err| {
            format!("error reading environment variable {var_name} (needed for default value of field {field_name}): {err}")
        })
    }
}

impl Reporter for SlackWebhookReporter {
    type Err = crate::Error;

    #[instrument(skip(self, _previous_leaderboard, leaderboard, changes), err)]
    async fn report_changes(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        _previous_leaderboard: &Leaderboard,
        leaderboard: &Leaderboard,
        changes: &Changes,
    ) -> Result<(), Self::Err> {
        let message = WebhookMessage::builder()
            .channel(self.channel.clone())
            .username(self.username.clone())
            .icon_url(self.icon_url.clone())
            .text(self.message_text(leaderboard, changes))
            .build()?;
        trace!(?message);

        let response = self
            .http_client
            .post(&self.webhook_url)
            .json(&message)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);
        match response {
            Ok(_) => Ok(()),
            Err(source) => Err(WebhookError::ReportChangesError {
                year,
                leaderboard_id,
                webhook_url: self.webhook_url.clone(),
                channel: self.channel.clone(),
                source,
            }
            .into()),
        }
    }

    #[instrument(skip(self, error))]
    async fn report_error<S>(&mut self, year: i32, leaderboard_id: u64, error: S)
    where
        S: Into<String> + Send,
    {
        let error = error.into();
        error!("aoc_leaderbot error for leaderboard {leaderboard_id} and year {year}: {error}");

        let message = WebhookMessage::builder()
            .channel(self.channel.clone())
            .username(self.username.clone())
            .icon_url(self.icon_url.clone())
            .text(format!(
                "An error occurred while trying to look for leaderboard changes: {error}"
            ))
            .build();
        match message {
            Ok(message) => {
                let response = self.http_client
                    .post(&self.webhook_url)
                    .json(&message)
                    .send()
                    .await
                    .and_then(reqwest::Response::error_for_status);
                if let Err(err) = response {
                    error!("error trying to report previous error to Slack webhook for leaderboard {leaderboard_id} and year {year}: {err}");
                }
            },
            Err(err) => error!("error trying to build webhook error message for leaderboard {leaderboard_id} and year {year}: {err}"),
        }
    }
}
