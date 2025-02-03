//! Custom error type definition.

use crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError;

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
    Builder(#[from] SlackWebhookReporterBuilderError),
}

impl From<SlackWebhookReporterBuilderError> for Error {
    fn from(value: SlackWebhookReporterBuilderError) -> Self {
        WebhookError::from(value).into()
    }
}
