//! Custom error type definition.

use std::env;
use std::ffi::OsString;
use std::num::ParseIntError;

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Attempted to build an instance of a type via a builder, but
    /// a required field was missing.
    #[error("missing field {field} to build a {target}")]
    MissingField {
        /// Type of value that was supposed to be built.
        target: &'static str,

        /// Name of missing field.
        field: &'static str,
    },

    /// Error while getting the value of an environment variable.
    #[error("error fetching environment variable {var_name}: {source}")]
    Env {
        /// Name of environment variable.
        var_name: String,

        /// Error that occurred while trying to get environment variable's value.
        source: EnvVarError,
    },

    /// Error while fetching leaderboard data from the AoC website.
    #[error(transparent)]
    Leaderboard(#[from] aoc_leaderboard::Error),
}

/// A version of [`env::VarError`] with additional variants.
#[derive(Debug, thiserror::Error)]
pub enum EnvVarError {
    /// Environment variable is not present.
    ///
    /// Our equivalent of [`env::VarError::NotPresent`].
    #[error("variable not present in environment")]
    NotPresent,

    /// Environment variable contains invalid, non-Unicode characters.
    ///
    /// Our equivalent of [`env::VarError::NotUnicode`].
    #[error("variable contained invalid, non-Unicode characters")]
    NotUnicode(OsString),

    /// Environment variable was expected to contain an integer value but didn't.
    #[error("expected int value, found {actual}: {source}")]
    IntExpected {
        /// The actual content of the environment variable.
        actual: String,

        /// The error that occurred while parsing the environment variable's content.
        source: ParseIntError,
    },
}

impl From<env::VarError> for EnvVarError {
    fn from(value: env::VarError) -> Self {
        match value {
            env::VarError::NotPresent => EnvVarError::NotPresent,
            env::VarError::NotUnicode(value) => EnvVarError::NotUnicode(value),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    mod from_env_var_error_for_env_var_error {
        use super::*;

        #[test]
        fn test_not_present() {
            let err = env::VarError::NotPresent;
            let actual: EnvVarError = err.into();

            assert_matches!(actual, EnvVarError::NotPresent);
        }

        #[test]
        fn test_not_unicode() {
            let err = env::VarError::NotUnicode("foo".into());
            let actual: EnvVarError = err.into();

            assert_matches!(actual, EnvVarError::NotUnicode(value) if value == "foo");
        }
    }
}
