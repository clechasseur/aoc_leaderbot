//! Implementations of [`LeaderbotConfig`](crate::leaderbot::LeaderbotConfig).

#[cfg(feature = "config-static")]
use chrono::{Datelike, Local};

/// Static bot config with predefined values.
#[cfg(feature = "config-static")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static")))]
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_builder::Builder,
)]
#[builder(
    derive(Debug, PartialEq, Eq, Hash),
    build_fn(name = "build_internal", error = "derive_builder::UninitializedFieldError", private)
)]
#[builder_struct_attr(cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static"))))]
#[builder_impl_attr(cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static"))))]
pub struct StaticLeaderbotConfig {
    /// Year for which to monitor the leaderboard.
    ///
    /// If not provided, the current year will be used.
    #[builder(default = "Local::now().year()")]
    pub year: i32,

    /// Leaderboard ID.
    ///
    /// See [`LeaderbotConfig::leaderboard_id`] for info on this value.
    ///
    /// [`LeaderbotConfig::leaderboard_id`]: crate::leaderbot::LeaderbotConfig::leaderboard_id
    pub leaderboard_id: u64,

    /// AoC session token.
    ///
    /// See [`LeaderbotConfig::aoc_session`] for info on this value.
    ///
    /// [`LeaderbotConfig::aoc_session`]: crate::leaderbot::LeaderbotConfig::aoc_session
    #[builder(setter(into))]
    pub aoc_session: String,
}

#[cfg(feature = "config-static")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static")))]
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

#[cfg(feature = "config-static")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static")))]
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
                target: std::any::type_name::<StaticLeaderbotConfig>(),
                field: field_err.field_name(),
            }),
        }
    }
}

#[cfg(feature = "config-static")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-static")))]
impl crate::leaderbot::LeaderbotConfig for StaticLeaderbotConfig {
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

/// Loads bot config values from the environment.
///
/// The following environment variables are used:
///
/// | Env var name             | Config field     | Default value |
/// |--------------------------|------------------|---------------|
/// | `{prefix}YEAR`           | `year`           | Current year  |
/// | `{prefix}LEADERBOARD_ID` | `leaderboard_id` | -             |
/// | `{prefix}AOC_SESSION`    | `aoc_session`    | -             |
#[cfg(feature = "config-env")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-env")))]
pub fn get_env_config<S>(
    env_var_prefix: S,
) -> crate::Result<impl crate::leaderbot::LeaderbotConfig + Send>
where
    S: AsRef<str>,
{
    let env_var_prefix = env_var_prefix.as_ref();
    let var_name = |name| format!("{env_var_prefix}{name}");

    let year = match crate::detail::int_env_var(var_name("YEAR")) {
        Ok(year) => Some(year),
        Err(crate::Error::Env(crate::error::EnvVarError::NotPresent)) => None,
        Err(err) => return Err(err),
    };

    let mut builder = StaticLeaderbotConfig::builder();
    if let Some(year) = year {
        builder.year(year);
    }
    builder
        .leaderboard_id(crate::detail::int_env_var(var_name("LEADERBOARD_ID"))?)
        .aoc_session(std::env::var(var_name("AOC_SESSION"))?)
        .build()
}
