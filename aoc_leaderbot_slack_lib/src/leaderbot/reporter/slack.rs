//! Implementations of [`leaderbot::Reporter`](aoc_leaderbot_lib::leaderbot::Reporter) for Slack.

pub mod webhook;

/// Default username used when posting messages to Slack.
pub const DEFAULT_USERNAME: &str = "Advent of Code";

/// Default icon URL used when posting messages to Slack.
pub const DEFAULT_ICON_URL: &str = "https://raw.githubusercontent.com/clechasseur/aoc_leaderbot/31004b36b4cff53ba4d741734f6db45d338708e5/resources/aoc_xmas.png";

/// User agent used to send requests to Slack.
pub const USER_AGENT: &str = concat!("aoc_leaderbot_slack@", env!("CARGO_PKG_VERSION"));
