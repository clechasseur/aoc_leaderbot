//! Helpers pertaining to [Slack webhooks](https://api.slack.com/messaging/webhooks).

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Content of a message that can be sent to a [Slack webhook].
///
/// [Slack webhook]: https://api.slack.com/messaging/webhooks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
#[builder(
    derive(Debug, PartialEq, Eq, Hash),
    setter(into, strip_option),
    build_fn(private, name = "build_internal")
)]
pub struct WebhookMessage {
    /// Name of Slack channel to post the message to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,

    /// Username to use when posting the message to Slack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// URL of an icon to use for the Slack user posting the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    /// Message text content.
    pub text: String,
}

impl WebhookMessage {
    /// Creates a [builder](WebhookMessageBuilder) to help create
    /// a new webhook message.
    pub fn builder() -> WebhookMessageBuilder {
        WebhookMessageBuilder::default()
    }
}

impl WebhookMessageBuilder {
    /// Builds the [`WebhookMessage`].
    pub fn build(&self) -> crate::Result<WebhookMessage> {
        self.build_internal().map_err(Into::into)
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(&self) -> Result<WebhookMessage, WebhookMessageBuilderError> {
        self.build_internal()
    }
}
