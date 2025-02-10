//! Slack-related helpers.

#[cfg(feature = "webhook-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "webhook-base")))]
pub mod webhook;
