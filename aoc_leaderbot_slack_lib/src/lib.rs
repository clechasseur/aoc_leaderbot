//! Library implementing Slack-specific functionalities for [`aoc_leaderbot`], a bot that can watch
//! an [Advent of Code] private leaderboard for changes and report them to various channels
//! like Slack.
//!
//! ## Trait implementations
//!
//! This library includes implementations of the traits found in [`aoc_leaderbot_lib`].
//!
//! ### [`SlackWebhookReporter`]
//!
//! Required feature: `reporter-webhook` (enabled by default)
//!
//! An implementation of the [`Reporter`] trait that reports changes to the leaderboard to a
//! Slack channel via a [Slack webhook].
//!
//! The reporter has several configurable input properties.  Although most have default values,
//! at least two must be specified explicitly:
//!
//! * [`webhook_url`]: URL of the Slack webhook to use to report changes.
//! * [`channel`]: Slack channel where to post message reporting changes.
//!
//! There are other optional properties that can be specified. The easiest way to create a
//! reporter instance would be via the [`builder`].  Many properties will also default to reading
//! their values from environment variables (see each property's documentation for details).
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [`SlackWebhookReporter`]: leaderbot::reporter::slack::webhook::SlackWebhookReporter
//! [`Reporter`]: aoc_leaderbot_lib::leaderbot::Reporter
//! [Slack webhook]: https://api.slack.com/messaging/webhooks
//! [`webhook_url`]: leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilder::webhook_url
//! [`channel`]: leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilder::channel
//! [`builder`]: leaderbot::reporter::slack::webhook::SlackWebhookReporter::builder

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod error;
pub mod leaderbot;
pub mod slack;

pub use error::Error;
pub use error::Result;
#[cfg(feature = "reporter-webhook")]
#[doc(hidden)]
pub use reqwest;
