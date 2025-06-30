//! Bot config loading values from the environment.

use std::fmt::Debug;

use crate::detail::{env_var, int_env_var};
use crate::error::EnvVarError;
use crate::leaderbot::Config;
use crate::leaderbot::config::mem::MemoryConfig;

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
#[cfg_attr(not(coverage_nightly), tracing::instrument(level = "trace", err))]
pub fn get_env_config<S>(env_var_prefix: S) -> crate::Result<impl Config + Send + Debug>
where
    S: AsRef<str> + Debug,
{
    let env_var_prefix = env_var_prefix.as_ref();
    let var_name = |name| format!("{env_var_prefix}{name}");

    let year = match int_env_var(var_name(ENV_CONFIG_YEAR_SUFFIX)) {
        Ok(year) => Some(year),
        Err(crate::Error::Env { source: EnvVarError::NotPresent, .. }) => None,
        Err(err) => return Err(err),
    };

    let mut builder = MemoryConfig::builder();
    if let Some(year) = year {
        builder.year(year);
    }
    builder
        .leaderboard_id(int_env_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX))?)
        .aoc_session(env_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX))?)
        .build()
}
