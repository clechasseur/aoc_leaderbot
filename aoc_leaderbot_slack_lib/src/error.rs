//! Custom error type definition.

use crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError;
use crate::slack::webhook::WebhookMessageBuilderError;

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error related to a Slack webhook.
    #[error(transparent)]
    Webhook(#[from] WebhookError),
}

/// Error type used for problems related to Slack webhooks.
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    /// Error returned when failing to build a [`SlackWebhookReporter`].
    ///
    /// [`SlackWebhookReporter`]: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporter
    #[error("error building Slack webhook reporter: {0}")]
    ReporterBuilder(#[from] SlackWebhookReporterBuilderError),

    /// An error occurred while trying to report leaderboard changes to a Slack webhook.
    #[error(
        "error reporting changes to leaderboard id {leaderboard_id} for year {year}: {source}"
    )]
    ReportChangesError {
        /// Year of leaderboard that changed.
        year: i32,

        /// ID of leaderboard that changed.
        leaderboard_id: u64,

        /// URL of Slack webhook where we tried to report changes.
        webhook_url: String,

        /// Slack channel where we tried to report changes.
        channel: String,

        /// HTTP error that occurred when trying to report changes.
        source: reqwest::Error,
    },

    /// Error returned when failing to build a [`WebhookMessage`].
    ///
    /// [`WebhookMessage`]: crate::slack::webhook::WebhookMessage
    #[error("error building Slack webhook message: {0}")]
    MessageBuilder(#[from] WebhookMessageBuilderError),
}

impl From<SlackWebhookReporterBuilderError> for Error {
    fn from(value: SlackWebhookReporterBuilderError) -> Self {
        WebhookError::from(value).into()
    }
}

impl From<WebhookMessageBuilderError> for Error {
    fn from(value: WebhookMessageBuilderError) -> Self {
        WebhookError::from(value).into()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serial_test::serial;

    use super::*;

    mod from_slack_webhook_reporter_builder_error_for_error {
        use std::env;

        use super::*;
        use crate::leaderbot::reporter::slack::webhook::{
            SlackWebhookReporter, WEBHOOK_URL_ENV_VAR,
        };

        #[test]
        #[serial(slack_webhook_reporter_env)]
        fn reporter_builder() {
            env::remove_var(WEBHOOK_URL_ENV_VAR);

            let error = SlackWebhookReporter::builder()
                .build_for_test()
                .unwrap_err();
            let error: Error = error.into();
            assert_matches!(error, Error::Webhook(WebhookError::ReporterBuilder(_)));
        }
    }

    mod from_webhook_message_builder_error_for_error {
        use super::*;
        use crate::slack::webhook::WebhookMessage;

        #[test]
        fn message_builder() {
            let error = WebhookMessage::builder().build_for_test().unwrap_err();
            let error: Error = error.into();
            assert_matches!(error, Error::Webhook(WebhookError::MessageBuilder(_)));
        }
    }
}
