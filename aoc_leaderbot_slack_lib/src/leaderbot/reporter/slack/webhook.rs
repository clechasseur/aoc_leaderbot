//! Implementations of [`Reporter`] using Slack webhooks.

use std::env;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderbot_lib::leaderbot::{Changes, Reporter};
use derive_builder::Builder;
use reqwest::Client;

use crate::leaderbot::reporter::slack::USER_AGENT;

/// Environment variable from which the Slack webhook URL will be
/// fetched if not specified.
pub const WEBHOOK_URL_ENV_VAR: &str = "SLACK_WEBHOOK_URL";

/// Environment variable from which the Slack channel will be fetched
/// if not specified.
pub const CHANNEL_ENV_VAR: &str = "SLACK_CHANNEL";

/// An [`aoc_leaderbot`] [`Reporter`] that sends leaderboard updates
/// to a Slack channel via a Slack webhook URL.
///
/// [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(private, name = "build_internal"))]
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

    #[builder(private, default = "Self::default_http_client()?")]
    _client: Client,
}

impl SlackWebhookReporter {
    /// Returns a [builder](SlackWebhookReporterBuilder) that can be used
    /// to customize a Slack webhook reporter.
    pub fn builder() -> SlackWebhookReporterBuilder {
        SlackWebhookReporterBuilder::default()
    }
}

impl SlackWebhookReporterBuilder {
    /// Builds the [`SlackWebhookReporter`].
    pub fn build(&self) -> crate::Result<SlackWebhookReporter> {
        self.build_internal().map_err(Into::into)
    }

    fn default_webhook_url() -> Result<String, String> {
        Self::env_var(WEBHOOK_URL_ENV_VAR, "webhook_url")
    }

    fn default_channel() -> Result<String, String> {
        Self::env_var(CHANNEL_ENV_VAR, "channel")
    }

    fn default_http_client() -> Result<Client, String> {
        Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|err| format!("error building HTTP client: {err}"))
    }

    fn env_var(var_name: &str, field_name: &str) -> Result<String, String> {
        env::var(var_name).map_err(|err| {
            format!("error reading environment variable {var_name} (needed for default value of field {field_name}: {err}")
        })
    }
}

impl Reporter for SlackWebhookReporter {
    type Err = crate::Error;

    async fn report_changes(
        &mut self,
        _year: i32,
        _leaderboard_id: u64,
        _previous_leaderboard: &Leaderboard,
        _leaderboard: &Leaderboard,
        _changes: &Changes,
    ) -> Result<(), Self::Err> {
        todo!()
    }

    async fn report_error<S>(&mut self, _year: i32, _leaderboard_id: u64, _error: S)
    where
        S: Into<String> + Send,
    {
        todo!()
    }
}
