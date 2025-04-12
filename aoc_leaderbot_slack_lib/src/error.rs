//! Custom error type definition.

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error related to a Slack webhook.
    #[cfg(feature = "webhook-base")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "webhook-base")))]
    #[error(transparent)]
    Webhook(#[from] WebhookError),
}

/// Error type used for problems related to Slack webhooks.
#[cfg(feature = "webhook-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "webhook-base")))]
#[cfg_attr(feature = "reporter-webhook", derive(veil::Redact))]
#[cfg_attr(not(feature = "reporter-webhook"), derive(Debug))]
#[derive(thiserror::Error, gratte::EnumDiscriminants, gratte::EnumIs)]
#[non_exhaustive]
#[strum_discriminants(
    name(WebhookErrorKind),
    derive(serde::Serialize, serde::Deserialize, gratte::EnumIs),
    non_exhaustive
)]
pub enum WebhookError {
    /// Error returned when failing to build a [`SlackWebhookReporter`].
    ///
    /// [`SlackWebhookReporter`]: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporter
    #[cfg(feature = "reporter-webhook")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "reporter-webhook")))]
    #[error("error building Slack webhook reporter: {0}")]
    ReporterBuilder(
        #[from] crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError,
    ),

    /// An error occurred while trying to report leaderboard changes to a Slack webhook.
    #[cfg(feature = "reporter-webhook")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "reporter-webhook")))]
    #[error(
        "error reporting changes to leaderboard id {leaderboard_id} for year {year}: {source}"
    )]
    ReportChanges {
        /// Year of leaderboard that changed.
        year: i32,

        /// ID of leaderboard that changed.
        leaderboard_id: u64,

        /// URL of Slack webhook where we tried to report changes.
        #[redact(partial)]
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
    MessageBuilder(#[from] crate::slack::webhook::WebhookMessageBuilderError),
}

#[cfg(feature = "reporter-webhook")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "reporter-webhook")))]
impl WebhookError {
    /// Returns `true` if the enum is [`WebhookError::ReporterBuilder`] and the internal
    /// [`SlackWebhookReporterBuilderError`] matches the given predicate.
    ///
    /// [`SlackWebhookReporterBuilderError`]: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError
    pub fn is_reporter_builder_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError) -> bool,
    {
        match self {
            Self::ReporterBuilder(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`WebhookError::ReportChanges`] and the error
    /// parameters match the given predicate.
    pub fn is_report_changes_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(i32, u64, &str, &str, &reqwest::Error) -> bool,
    {
        match self {
            Self::ReportChanges { year, leaderboard_id, webhook_url, channel, source } => {
                predicate(*year, *leaderboard_id, webhook_url, channel, source)
            },
            _ => false,
        }
    }
}

#[cfg(feature = "webhook-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "webhook-base")))]
impl WebhookError {
    /// Returns `true` if the enum is [`WebhookError::MessageBuilder`] and the internal
    /// [`WebhookMessageBuilderError`](crate::slack::webhook::WebhookMessageBuilderError)
    /// matches the given predicate.
    pub fn is_message_builder_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&crate::slack::webhook::WebhookMessageBuilderError) -> bool,
    {
        match self {
            Self::MessageBuilder(source) => predicate(source),
            _ => false,
        }
    }
}

#[cfg(feature = "reporter-webhook")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "reporter-webhook")))]
impl From<crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError> for Error {
    fn from(
        value: crate::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError,
    ) -> Self {
        WebhookError::from(value).into()
    }
}

#[cfg(feature = "webhook-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "webhook-base")))]
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
