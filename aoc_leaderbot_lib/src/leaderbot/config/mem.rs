//! Memory-based bot config implementation

use std::any::type_name;

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

    /// AoC session token.
    ///
    /// See [`Config::aoc_session`] for info on this value.
    #[builder(setter(into))]
    pub aoc_session: String,
}

impl MemoryConfig {
    /// Creates a builder to initialize a new instance.
    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::default()
    }

    /// Creates a new instance with values for all fields.
    pub fn new<S>(year: i32, leaderboard_id: u64, aoc_session: S) -> Self
    where
        S: Into<String>,
    {
        Self { year, leaderboard_id, aoc_session: aoc_session.into() }
    }
}

impl MemoryConfigBuilder {
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
    fn year(&self) -> i32 {
        self.year
    }

    fn leaderboard_id(&self) -> u64 {
        self.leaderboard_id
    }

    fn aoc_session(&self) -> String {
        self.aoc_session.clone()
    }
}
