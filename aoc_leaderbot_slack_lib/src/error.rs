//! Custom error type definition.

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error related to a Slack webhook.
    #[cfg(feature = "webhook-base")]
    #[error(transparent)]
    Webhook(#[from] WebhookError),
}

/// Error type used for problems related to Slack webhooks.
#[cfg(feature = "webhook-base")]
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    /// Error returned when failing to build a [`SlackWebhookReporter`].
    ///
    /// [`SlackWebhookReporter`]: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporter
    #[cfg(feature = "reporter-webhook")]
    #[error("error building Slack webhook reporter: {0}")]
    ReporterBuilder(
        #[from] crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError,
    ),

    /// An error occurred while trying to report leaderboard changes to a Slack webhook.
    #[cfg(feature = "reporter-webhook")]
    #[error("error reporting changes to Slack: {0}")]
    ReportChanges(WebhookMessageError),

    /// An error occurred while trying to
    #[cfg(feature = "reporter-webhook")]
    #[error("error reporting first bot run to Slack: {0}")]
    ReportFirstRun(WebhookMessageError),

    /// Error returned when failing to build a [`WebhookMessage`].
    ///
    /// [`WebhookMessage`]: crate::slack::webhook::WebhookMessage
    #[error("error building Slack webhook message: {0}")]
    MessageBuilder(#[from] crate::slack::webhook::WebhookMessageBuilderError),
}

/// Content of an error that occurred while sending a message to a Slack webhook.
#[cfg(feature = "reporter-webhook")]
#[derive(veil::Redact, thiserror::Error)]
#[error(
    "error sending message to Slack about leaderboard id {leaderboard_id} for year {year} in channel #{channel}: {source}"
)]
pub struct WebhookMessageError {
    /// Year of leaderboard.
    pub year: i32,

    /// ID of leaderboard.
    pub leaderboard_id: u64,

    /// URL of Slack webhook where we tried to send the message.
    #[redact(partial)]
    pub webhook_url: String,

    /// Slack channel where we tried to send the message.
    pub channel: String,

    /// HTTP error that occurred when trying to send the message.
    pub source: reqwest::Error,
}

#[cfg(feature = "reporter-webhook")]
impl From<crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError> for Error {
    fn from(
        value: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError,
    ) -> Self {
        WebhookError::from(value).into()
    }
}

#[cfg(feature = "webhook-base")]
impl From<crate::slack::webhook::WebhookMessageBuilderError> for Error {
    fn from(value: crate::slack::webhook::WebhookMessageBuilderError) -> Self {
        WebhookError::from(value).into()
    }
}

#[cfg(all(test, feature = "webhook-base"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[cfg(feature = "reporter-webhook")]
    mod from_slack_webhook_reporter_builder_error_for_error {
        use std::env;

        use serial_test::serial;

        use super::*;
        use crate::leaderbot::reporter::slack::webhook::{
            SlackWebhookReporter, WEBHOOK_URL_ENV_VAR,
        };

        #[test]
        #[serial(slack_webhook_reporter_env)]
        fn reporter_builder() {
            unsafe {
                env::remove_var(WEBHOOK_URL_ENV_VAR);
            }

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
