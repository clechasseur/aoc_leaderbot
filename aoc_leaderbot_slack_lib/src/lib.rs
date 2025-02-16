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
//! Although most have default values, the reporter supports several customizable fields.
//! The easiest way to create one would be via the [`builder`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/struct.SlackWebhookReporterBuilder.html).
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [`SlackWebhookReporter`]: leaderbot::reporter::slack::webhook::SlackWebhookReporter
//! [`Reporter`]: aoc_leaderbot_lib::leaderbot::Reporter
//! [Slack webhook]: https://api.slack.com/messaging/webhooks

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(any(nightly_rustc, docsrs), feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod error;
pub mod leaderbot;
pub mod slack;

pub use error::Error;
pub use error::Result;
#[cfg(feature = "reporter-webhook")]
#[doc(hidden)]
pub use reqwest;
