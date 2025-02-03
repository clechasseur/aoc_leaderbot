//! Implementations of [`leaderbot::Reporter`](aoc_leaderbot_lib::leaderbot::Reporter) for Slack.

pub mod webhook;

/// Default username used when posting messages to Slack.
pub const DEFAULT_USERNAME: &str = "Advent of Code";

/// Default icon URL used when posting messages to Slack.
pub const DEFAULT_ICON_URL: &str = "<TODO>";

/// User agent used to send requests to Slack.
pub const USER_AGENT: &str = concat!("aoc_leaderbot@", env!("CARGO_PKG_VERSION"));
