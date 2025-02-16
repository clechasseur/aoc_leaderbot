//! Library implementing the core functionalities of [`aoc_leaderbot`], a bot that can watch
//! an [Advent of Code] private leaderboard for changes and report them to various channels
//! like Slack.
//!
//! ## Usage
//!
//! The bot's body is implemented via the [`run_bot`] function. This function will fetch the
//! current version of the AoC leaderboard, then check if we had a previous  version (from an
//! earlier run).  If no previous version exists, the bot saves the current version and exists.
//! Otherwise, the bot compare the current leaderboard data with the previous one.  If there are
//! new members or if existing members got new stars, it reports changes and saves the current
//! version as the last one seen.
//!
//! In order to function, the bot needs three things, which are passed using traits.
//!
//! ### [`Config`]
//!
//! This trait is used by the bot to fetch information about what AoC leaderboard to watch.
//! It is a read-only trait providing three pieces of information: the [leaderboard ID], the
//! [AoC session token] and the [year]. The latter defaults to the current year.
//!
//! ### [`Storage`]
//!
//! This trait abstracts the bot's storage facility.  It is used to load leaderboard data from a
//! previous run and to save any new leaderboard data.
//!
//! ### [`Reporter`]
//!
//! This trait abstracts the bot's capability to report leaderboard changes when it finds some.
//! Its main purpose is to implement the [`report_changes`] method to report changes to the user.
//! This could be via a Slack post, a Discord message, etc.
//!
//! The reporter can also be used to report any error occurring during bot execution (ex: expired
//! AoC session token, etc.) via its [`report_error`] method.
//!
//! ## Concrete implementations
//!
//! Although this library includes the bot's core function, it does not provide all possible
//! implementations of the traits it needs for operations. This library includes two implementations
//! of [`Config`], one implementation of [`Storage`] and no implementation of [`Reporter`]. Users
//! will thus need to implement a [`Reporter`] at a minimum.
//!
//! For other trait implementations, you can look at related crates like [`aoc_leaderbot_slack_lib`].
//!
//! ### [`MemoryConfig`]
//!
//! Required feature: `config-mem` (enabled by default)
//!
//! This implementation of [`Config`] simply stores its values in memory.  The most basic
//! implementation, but it works.
//!
//! ### [`get_env_config`]
//!
//! Required feature: `config-env` (enabled by default)
//!
//! This function returns an opaque [`Config`] implementation fetching the parameters from
//! environment variables.  This is possibly the most flexible way of providing the bot its
//! config, which is why it is enabled by default.
//!
//! ### [`MemoryStorage`]
//!
//! Required feature: `storage-mem`
//!
//! This implementation of [`Storage`] simply stores its data in memory. Although this means that
//! it would technically lose its data upon program exit, the whole storage can be persisted using
//! [`serde`], which means it's a possibly-decent implementation.
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [`run_bot`]: leaderbot::run_bot
//! [`Config`]: leaderbot::Config
//! [leaderboard ID]: leaderbot::Config::leaderboard_id
//! [AoC session token]: leaderbot::Config::aoc_session
//! [year]: leaderbot::Config::year
//! [`Storage`]: leaderbot::Storage
//! [`Reporter`]: leaderbot::Reporter
//! [`report_changes`]: leaderbot::Reporter::report_changes
//! [`report_error`]: leaderbot::Reporter::report_error
//! [`aoc_leaderbot_slack_lib`]: https://crates.io/crates/aoc_leaderbot_slack_lib
//! [`MemoryConfig`]: leaderbot::config::mem::MemoryConfig
//! [`get_env_config`]: leaderbot::config::env::get_env_config
//! [`MemoryStorage`]: leaderbot::storage::mem::MemoryStorage
//! [`serde`]: https://serde.rs/

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(any(nightly_rustc, docsrs), feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub(crate) mod detail;
pub mod error;
pub mod leaderbot;
pub(crate) mod mockable;

pub use error::Error;
pub use error::Result;
#[mockall_double::double]
pub(crate) use mockable::helpers as mockable_helpers;
