//! Memory-based bot config implementation

use std::any::type_name;

use aoc_leaderboard::aoc::LeaderboardCredentials;
use chrono::{Datelike, Local};
use derive_builder::{Builder, UninitializedFieldError};
use serde::{Deserialize, Serialize};

use crate::leaderbot::Config;

/// Bot config storing values in memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
#[builder(
    derive(Debug, PartialEq, Eq, Hash),
    build_fn(name = "build_internal", error = "UninitializedFieldError", private)
)]
pub struct MemoryConfig {
    /// Year for which to monitor the leaderboard.
    ///
    /// If not provided, the current year will be used.
    #[builder(default = "Local::now().year()")]
    pub year: i32,

    /// Leaderboard ID.
    ///
    /// See [`Config::leaderboard_id`] for info on this value.
    pub leaderboard_id: u64,

    /// AoC leaderboard credentials.
    ///
    /// See [`Config::credentials`] for info on this value.
    #[builder(setter(into))]
    pub credentials: LeaderboardCredentials,
}

impl MemoryConfig {
    /// Creates a builder to initialize a new instance.
    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::default()
    }

    /// Creates a new instance with values for all fields.
    pub fn new(year: i32, leaderboard_id: u64, credentials: LeaderboardCredentials) -> Self {
        Self { year, leaderboard_id, credentials }
    }
}

impl MemoryConfigBuilder {
    /// Sets the config's [`credentials`] to the given [view key].
    ///
    /// [`credentials`]: MemoryConfig::credentials
    /// [view key]: LeaderboardCredentials::ViewKey
    pub fn view_key<S>(&mut self, view_key: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.credentials = Some(LeaderboardCredentials::ViewKey(view_key.into()));
        self
    }

    /// Sets the config's [`credentials`] to the given [session cookie].
    ///
    /// [`credentials`]: MemoryConfig::credentials
    /// [session cookie]: LeaderboardCredentials::SessionCookie
    pub fn session_cookie<S>(&mut self, session_cookie: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.credentials = Some(LeaderboardCredentials::SessionCookie(session_cookie.into()));
        self
    }

    /// Builds a new [`MemoryConfig`].
    ///
    /// # Errors
    ///
    /// - [`Error::MissingField`]: if a required field was not specified
    ///
    /// [`Error::MissingField`]: crate::error::Error::MissingField
    pub fn build(&self) -> crate::Result<MemoryConfig> {
        match self.build_internal() {
            Ok(config) => Ok(config),
            Err(field_err) => Err(crate::Error::MissingField {
                target: type_name::<MemoryConfig>(),
                field: field_err.field_name(),
            }),
        }
    }
}

impl Config for MemoryConfig {
    #[cfg_attr(not(coverage), tracing::instrument(skip(self), level = "trace", ret))]
    fn year(&self) -> i32 {
        self.year
    }

    #[cfg_attr(not(coverage), tracing::instrument(skip(self), level = "trace", ret))]
    fn leaderboard_id(&self) -> u64 {
        self.leaderboard_id
    }

    #[cfg_attr(not(coverage), tracing::instrument(skip(self), level = "trace", ret))]
    fn credentials(&self) -> LeaderboardCredentials {
        self.credentials.clone()
    }
}
