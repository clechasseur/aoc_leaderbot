//! Implementations of [`LeaderbotConfig`](crate::leaderbot::LeaderbotConfig).

pub use config_static::*;
pub use config_env::*;

#[cfg(feature = "config-static")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static")))]
mod config_static {
    use std::any::type_name;
    use chrono::{Datelike, Local};
    use derive_builder::{Builder, UninitializedFieldError};
    use serde::{Deserialize, Serialize};
    use crate::leaderbot::LeaderbotConfig;

    /// Static bot config with predefined values.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        derive(Debug, PartialEq, Eq, Hash),
        build_fn(name = "build_internal", error = "UninitializedFieldError", private)
    )]
    pub struct StaticLeaderbotConfig {
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

    impl StaticLeaderbotConfig {
        /// Creates a builder to initialize a new instance.
        pub fn builder() -> StaticLeaderbotConfigBuilder {
            StaticLeaderbotConfigBuilder::default()
        }

        /// Creates a new instance with values for all fields.
        pub fn new<S>(year: i32, leaderboard_id: u64, aoc_session: S) -> Self
        where
            S: Into<String>,
        {
            Self { year, leaderboard_id, aoc_session: aoc_session.into() }
        }
    }

    impl StaticLeaderbotConfigBuilder {
        /// Builds a new [`StaticLeaderbotConfig`].
        ///
        /// # Errors
        ///
        /// - [`Error::MissingField`]: if a required field was not specified
        ///
        /// [`Error::MissingField`]: crate::error::Error::MissingField
        pub fn build(&self) -> crate::Result<StaticLeaderbotConfig> {
            match self.build_internal() {
                Ok(config) => Ok(config),
                Err(field_err) => Err(crate::Error::MissingField {
                    target: type_name::<StaticLeaderbotConfig>(),
                    field: field_err.field_name(),
                }),
            }
        }
    }

    impl LeaderbotConfig for StaticLeaderbotConfig {
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

#[cfg(feature = "config-env")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-env")))]
mod config_env {
    use std::env;
    use super::*;
    
    use crate::detail::int_env_var;
    use crate::error::EnvVarError;
    use crate::leaderbot::LeaderbotConfig;

    /// Loads bot config values from the environment.
    ///
    /// The following environment variables are used:
    ///
    /// | Env var name             | Config field     | Default value |
    /// |--------------------------|------------------|---------------|
    /// | `{prefix}YEAR`           | `year`           | Current year  |
    /// | `{prefix}LEADERBOARD_ID` | `leaderboard_id` | -             |
    /// | `{prefix}AOC_SESSION`    | `aoc_session`    | -             |
    pub fn get_env_config<S>(env_var_prefix: S) -> crate::Result<impl LeaderbotConfig + Send>
    where
        S: AsRef<str>,
    {
        let env_var_prefix = env_var_prefix.as_ref();
        let var_name = |name| format!("{env_var_prefix}{name}");

        let year = match int_env_var(var_name("YEAR")) {
            Ok(year) => Some(year),
            Err(crate::Error::Env(EnvVarError::NotPresent)) => None,
            Err(err) => return Err(err),
        };

        let mut builder = StaticLeaderbotConfig::builder();
        if let Some(year) = year {
            builder.year(year);
        }
        builder
            .leaderboard_id(int_env_var(var_name("LEADERBOARD_ID"))?)
            .aoc_session(env::var(var_name("AOC_SESSION"))?)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;
    use assert_matches::assert_matches;
    use chrono::{Datelike, Local};
    use crate::leaderbot::LeaderbotConfig;
    use super::*;

    #[cfg(feature = "config-static")]
    mod static_leaderbot_config {
        use super::*;
        
        #[test]
        fn new() {
            let actual = StaticLeaderbotConfig::new(2024, 12345, "aoc_session");

            assert_eq!(actual.year(), 2024);
            assert_eq!(actual.leaderboard_id(), 12345);
            assert_eq!(actual.aoc_session(), "aoc_session");
        }

        mod builder {
            use super::*;

            #[test]
            fn with_all_fields() {
                let actual = StaticLeaderbotConfig::builder()
                    .year(2024)
                    .leaderboard_id(12345)
                    .aoc_session("aoc_session")
                    .build()
                    .unwrap();

                assert_eq!(actual.year(), 2024);
                assert_eq!(actual.leaderboard_id(), 12345);
                assert_eq!(actual.aoc_session(), "aoc_session");
            }

            #[test]
            fn with_default_year() {
                let actual = StaticLeaderbotConfig::builder()
                    .leaderboard_id(12345)
                    .aoc_session("aoc_session")
                    .build()
                    .unwrap();

                assert_eq!(actual.year(), Local::now().year());
                assert_eq!(actual.leaderboard_id(), 12345);
                assert_eq!(actual.aoc_session(), "aoc_session");
            }

            #[test]
            fn with_missing_leaderboard_id() {
                let actual = StaticLeaderbotConfig::builder()
                    .year(2024)
                    .aoc_session("aoc_session")
                    .build();

                assert_matches!(actual, Err(crate::Error::MissingField { target, field }) => {
                    assert_eq!(target, type_name::<StaticLeaderbotConfig>());
                    assert_eq!(field, "leaderboard_id");
                });
            }

            #[test]
            fn with_missing_aoc_session() {
                let actual = StaticLeaderbotConfig::builder()
                    .year(2024)
                    .leaderboard_id(12345)
                    .build();

                assert_matches!(actual, Err(crate::Error::MissingField { target, field }) => {
                    assert_eq!(target, type_name::<StaticLeaderbotConfig>());
                    assert_eq!(field, "aoc_session");
                });
            }
        }
    }
}
