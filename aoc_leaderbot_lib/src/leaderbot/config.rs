//! Implementations of [`LeaderbotConfig`](crate::leaderbot::LeaderbotConfig).

/// Memory-based bot config implementation
#[cfg(feature = "config-mem")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-mem")))]
pub mod mem {
    use std::any::type_name;

    use chrono::{Datelike, Local};
    use derive_builder::{Builder, UninitializedFieldError};
    use serde::{Deserialize, Serialize};

    use crate::leaderbot::LeaderbotConfig;

    /// Bot config storing values in memory.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        derive(Debug, PartialEq, Eq, Hash),
        build_fn(name = "build_internal", error = "UninitializedFieldError", private)
    )]
    pub struct MemoryLeaderbotConfig {
        /// Year for which to monitor the leaderboard.
        ///
        /// If not provided, the current year will be used.
        #[builder(default = "Local::now().year()")]
        pub year: i32,

        /// Leaderboard ID.
        ///
        /// See [`LeaderbotConfig::leaderboard_id`] for info on this value.
        pub leaderboard_id: u64,

        /// AoC session token.
        ///
        /// See [`LeaderbotConfig::aoc_session`] for info on this value.
        #[builder(setter(into))]
        pub aoc_session: String,
    }

    impl MemoryLeaderbotConfig {
        /// Creates a builder to initialize a new instance.
        pub fn builder() -> MemoryLeaderbotConfigBuilder {
            MemoryLeaderbotConfigBuilder::default()
        }

        /// Creates a new instance with values for all fields.
        pub fn new<S>(year: i32, leaderboard_id: u64, aoc_session: S) -> Self
        where
            S: Into<String>,
        {
            Self { year, leaderboard_id, aoc_session: aoc_session.into() }
        }
    }

    impl MemoryLeaderbotConfigBuilder {
        /// Builds a new [`MemoryLeaderbotConfig`].
        ///
        /// # Errors
        ///
        /// - [`Error::MissingField`]: if a required field was not specified
        ///
        /// [`Error::MissingField`]: crate::error::Error::MissingField
        pub fn build(&self) -> crate::Result<MemoryLeaderbotConfig> {
            match self.build_internal() {
                Ok(config) => Ok(config),
                Err(field_err) => Err(crate::Error::MissingField {
                    target: type_name::<MemoryLeaderbotConfig>(),
                    field: field_err.field_name(),
                }),
            }
        }
    }

    impl LeaderbotConfig for MemoryLeaderbotConfig {
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
}

/// Bot config loading values from the environment
#[cfg(feature = "config-env")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-env")))]
pub mod env {
    use std::fmt::Debug;

    use crate::detail::{env_var, int_env_var};
    use crate::error::EnvVarError;
    use crate::leaderbot::config::mem::MemoryLeaderbotConfig;
    use crate::leaderbot::LeaderbotConfig;

    /// Environment variable name suffix for `year`. See [`get_env_config`].
    pub const ENV_CONFIG_YEAR_SUFFIX: &str = "YEAR";

    /// Environment variable name suffix for `leaderboard_id`. See [`get_env_config`].
    pub const ENV_CONFIG_LEADERBOARD_ID_SUFFIX: &str = "LEADERBOARD_ID";

    /// Environment variable name suffix for `aoc_session`. See [`get_env_config`].
    pub const ENV_CONFIG_AOC_SESSION_SUFFIX: &str = "AOC_SESSION";

    /// Loads bot config values from the environment.
    ///
    /// The following environment variables are used:
    ///
    /// | Env var name             | Config field     | Default value |
    /// |--------------------------|------------------|---------------|
    /// | `{prefix}YEAR`           | `year`           | Current year  |
    /// | `{prefix}LEADERBOARD_ID` | `leaderboard_id` | -             |
    /// | `{prefix}AOC_SESSION`    | `aoc_session`    | -             |
    pub fn get_env_config<S>(
        env_var_prefix: S,
    ) -> crate::Result<impl LeaderbotConfig + Send + Debug>
    where
        S: AsRef<str>,
    {
        let env_var_prefix = env_var_prefix.as_ref();
        let var_name = |name| format!("{env_var_prefix}{name}");

        let year = match int_env_var(var_name(ENV_CONFIG_YEAR_SUFFIX)) {
            Ok(year) => Some(year),
            Err(crate::Error::Env { source: EnvVarError::NotPresent, .. }) => None,
            Err(err) => return Err(err),
        };

        let mut builder = MemoryLeaderbotConfig::builder();
        if let Some(year) = year {
            builder.year(year);
        }
        builder
            .leaderboard_id(int_env_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX))?)
            .aoc_session(env_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX))?)
            .build()
    }
}
