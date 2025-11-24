//! Custom error type definition.

use std::env;
use std::ffi::{OsStr, OsString};
use std::num::ParseIntError;

use gratte::{EnumDiscriminants, EnumIs, IntoDiscriminant};
use serde::{Deserialize, Serialize};

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error, EnumIs)]
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

    /// Error while performing a [`Storage`] operation.
    ///
    /// [`Storage`]: crate::leaderbot::Storage
    #[error(transparent)]
    Storage(#[from] StorageError),

    /// Error while performing a [`Reporter`] operation.
    ///
    /// [`Reporter`]: crate::leaderbot::Reporter
    #[error(transparent)]
    Reporter(#[from] ReporterError),

    // The following errors are only used in tests, they will not be available to users.
    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestLoadPreviousError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestSaveUpdatedError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestSaveBaseError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestSaveErrorError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestReportChangesError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestReportFirstRunError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("something went wrong: {0}")]
    TestErrorWithMessage(String),
}

impl Error {
    /// Returns `true` if the enum is [`Error::MissingField`] and the target type and
    /// field name match the given predicate.
    pub fn is_missing_field_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&'static str, &'static str) -> bool,
    {
        match self {
            Self::MissingField { target, field } => predicate(target, field),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`Error::Env`] and the environment variable name
    /// and internal [`EnvVarError`] match the given predicate.
    pub fn is_env_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&str, &EnvVarError) -> bool,
    {
        match self {
            Error::Env { var_name, source } => predicate(var_name, source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`Error::Leaderboard`] and the internal
    /// [`aoc_leaderboard::Error`] matches the given predicate.
    pub fn is_leaderboard_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&aoc_leaderboard::Error) -> bool,
    {
        match self {
            Self::Leaderboard(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`Error::Storage`] and the internal [`StorageError`]
    /// matches the given predicate.
    pub fn is_storage_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&StorageError) -> bool,
    {
        match self {
            Self::Storage(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`Error::Reporter`] and the internal [`ReporterError`]
    /// matches the given predicate.
    pub fn is_reporter_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&ReporterError) -> bool,
    {
        match self {
            Self::Reporter(source) => predicate(source),
            _ => false,
        }
    }
}

/// A data-less equivalent to [`Error`], to store the kind of error
/// we encounter while running the bot.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIs)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Attempted to build an instance of a type via a builder, but
    /// a required field was missing.
    MissingField,

    /// Error while getting the value of an environment variable.
    Env(EnvVarErrorKind),

    /// Error while fetching leaderboard data from the AoC website.
    Leaderboard(aoc_leaderboard::ErrorKind),

    /// Error while performing a [`Storage`] operation.
    ///
    /// [`Storage`]: crate::leaderbot::Storage
    Storage(StorageErrorKind),

    /// Error while performing a [`Reporter`] operation.
    ///
    /// [`Reporter`]: crate::leaderbot::Reporter
    Reporter(ReporterErrorKind),

    // The following errors are only used in tests, they will not be available to users.
    #[cfg(test)]
    #[doc(hidden)]
    TestLoadPreviousError,

    #[cfg(test)]
    #[doc(hidden)]
    TestSaveUpdatedError,

    #[cfg(test)]
    #[doc(hidden)]
    TestSaveBaseError,

    #[cfg(test)]
    #[doc(hidden)]
    TestSaveErrorError,

    #[cfg(test)]
    #[doc(hidden)]
    TestReportChangesError,

    #[cfg(test)]
    #[doc(hidden)]
    TestReportFirstRunError,

    #[cfg(test)]
    #[doc(hidden)]
    TestErrorWithMessage,
}

impl ErrorKind {
    /// Returns `true` if the enum is [`ErrorKind::Env`] of the given [`EnvVarErrorKind`].
    pub fn is_env_of_kind(&self, env_var_error_kind: EnvVarErrorKind) -> bool {
        *self == ErrorKind::Env(env_var_error_kind)
    }

    /// Returns `true` if the enum is [`ErrorKind::Leaderboard`] of the given
    /// [`aoc_leaderboard::ErrorKind`].
    pub fn is_leaderboard_of_kind(
        &self,
        leaderboard_error_kind: aoc_leaderboard::ErrorKind,
    ) -> bool {
        *self == ErrorKind::Leaderboard(leaderboard_error_kind)
    }

    /// Returns `true` if the enum is [`ErrorKind::Storage`] of the given [`StorageErrorKind`].
    pub fn is_storage_of_kind(&self, storage_error_kind: StorageErrorKind) -> bool {
        *self == ErrorKind::Storage(storage_error_kind)
    }

    /// Returns `true` if the enum is [`ErrorKind::Reporter`] of the given [`ReporterErrorKind`].
    pub fn is_reporter_of_kind(&self, reporter_error_kind: ReporterErrorKind) -> bool {
        *self == ErrorKind::Reporter(reporter_error_kind)
    }
}

impl PartialEq<Error> for ErrorKind {
    fn eq(&self, other: &Error) -> bool {
        *self == ErrorKind::from(other)
    }
}

impl PartialEq<ErrorKind> for Error {
    fn eq(&self, other: &ErrorKind) -> bool {
        ErrorKind::from(self) == *other
    }
}

impl From<Error> for ErrorKind {
    fn from(value: Error) -> Self {
        (&value).into()
    }
}

impl From<&Error> for ErrorKind {
    fn from(value: &Error) -> Self {
        match value {
            Error::MissingField { .. } => ErrorKind::MissingField,
            Error::Env { source, .. } => ErrorKind::Env(source.into()),
            Error::Leaderboard(source) => ErrorKind::Leaderboard(source.into()),
            Error::Storage(source) => ErrorKind::Storage(source.into()),
            Error::Reporter(source) => ErrorKind::Reporter(source.into()),
            #[cfg(test)]
            Error::TestLoadPreviousError => ErrorKind::TestLoadPreviousError,
            #[cfg(test)]
            Error::TestSaveUpdatedError => ErrorKind::TestSaveUpdatedError,
            #[cfg(test)]
            Error::TestSaveBaseError => ErrorKind::TestSaveBaseError,
            #[cfg(test)]
            Error::TestSaveErrorError => ErrorKind::TestSaveErrorError,
            #[cfg(test)]
            Error::TestReportChangesError => ErrorKind::TestReportChangesError,
            #[cfg(test)]
            Error::TestReportFirstRunError => ErrorKind::TestReportFirstRunError,
            #[cfg(test)]
            Error::TestErrorWithMessage(_) => ErrorKind::TestErrorWithMessage,
        }
    }
}

impl IntoDiscriminant for Error {
    type Discriminant = ErrorKind;

    fn discriminant(&self) -> Self::Discriminant {
        self.into()
    }
}

/// A version of [`env::VarError`] with additional variants.
#[derive(Debug, thiserror::Error, EnumDiscriminants, EnumIs)]
#[strum_discriminants(name(EnvVarErrorKind), derive(Serialize, Deserialize, EnumIs))]
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

impl EnvVarError {
    /// Returns `true` if enum is [`EnvVarError::NotUnicode`] and the actual environment
    /// variable value matches the given predicate.
    pub fn is_not_unicode_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&OsStr) -> bool,
    {
        match self {
            Self::NotUnicode(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if enum is [`EnvVarError::IntExpected`] and the actual environment
    /// variable value and internal [`ParseIntError`] match the given predicate.
    pub fn is_int_expected_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&str, &ParseIntError) -> bool,
    {
        match self {
            Self::IntExpected { actual, source } => predicate(actual, source),
            _ => false,
        }
    }
}

impl PartialEq<EnvVarError> for env::VarError {
    fn eq(&self, other: &EnvVarError) -> bool {
        match (self, other) {
            (Self::NotPresent, EnvVarError::NotPresent) => true,
            (Self::NotUnicode(lhs), EnvVarError::NotUnicode(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl PartialEq<env::VarError> for EnvVarError {
    fn eq(&self, other: &env::VarError) -> bool {
        other == self
    }
}

impl PartialEq<EnvVarErrorKind> for EnvVarError {
    fn eq(&self, other: &EnvVarErrorKind) -> bool {
        EnvVarErrorKind::from(self) == *other
    }
}

impl PartialEq<EnvVarError> for EnvVarErrorKind {
    fn eq(&self, other: &EnvVarError) -> bool {
        *self == Self::from(other)
    }
}

impl PartialEq<ErrorKind> for EnvVarErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        ErrorKind::Env(*self) == *other
    }
}

impl PartialEq<EnvVarErrorKind> for ErrorKind {
    fn eq(&self, other: &EnvVarErrorKind) -> bool {
        *self == Self::Env(*other)
    }
}

impl PartialEq<Error> for EnvVarErrorKind {
    fn eq(&self, other: &Error) -> bool {
        *self == ErrorKind::from(other)
    }
}

impl PartialEq<EnvVarErrorKind> for Error {
    fn eq(&self, other: &EnvVarErrorKind) -> bool {
        ErrorKind::from(self) == *other
    }
}

impl PartialEq<env::VarError> for EnvVarErrorKind {
    fn eq(&self, other: &env::VarError) -> bool {
        *self == Self::from(other)
    }
}

impl PartialEq<EnvVarErrorKind> for env::VarError {
    fn eq(&self, other: &EnvVarErrorKind) -> bool {
        EnvVarErrorKind::from(self) == *other
    }
}

impl From<env::VarError> for EnvVarError {
    fn from(value: env::VarError) -> Self {
        match value {
            env::VarError::NotPresent => EnvVarError::NotPresent,
            env::VarError::NotUnicode(value) => EnvVarError::NotUnicode(value),
        }
    }
}

impl From<env::VarError> for EnvVarErrorKind {
    fn from(value: env::VarError) -> Self {
        (&value).into()
    }
}

impl From<&env::VarError> for EnvVarErrorKind {
    fn from(value: &env::VarError) -> Self {
        match value {
            env::VarError::NotPresent => EnvVarErrorKind::NotPresent,
            env::VarError::NotUnicode(_) => EnvVarErrorKind::NotUnicode,
        }
    }
}

impl From<EnvVarErrorKind> for ErrorKind {
    fn from(value: EnvVarErrorKind) -> Self {
        ErrorKind::Env(value)
    }
}

impl From<&EnvVarErrorKind> for ErrorKind {
    fn from(value: &EnvVarErrorKind) -> Self {
        (*value).into()
    }
}

impl From<EnvVarError> for ErrorKind {
    fn from(value: EnvVarError) -> Self {
        EnvVarErrorKind::from(value).into()
    }
}

impl From<&EnvVarError> for ErrorKind {
    fn from(value: &EnvVarError) -> Self {
        EnvVarErrorKind::from(value).into()
    }
}

impl From<env::VarError> for ErrorKind {
    fn from(value: env::VarError) -> Self {
        EnvVarErrorKind::from(value).into()
    }
}

impl From<&env::VarError> for ErrorKind {
    fn from(value: &env::VarError) -> Self {
        EnvVarErrorKind::from(value).into()
    }
}

impl From<aoc_leaderboard::ErrorKind> for ErrorKind {
    fn from(value: aoc_leaderboard::ErrorKind) -> Self {
        ErrorKind::Leaderboard(value)
    }
}

impl From<&aoc_leaderboard::ErrorKind> for ErrorKind {
    fn from(value: &aoc_leaderboard::ErrorKind) -> Self {
        (*value).into()
    }
}

impl From<aoc_leaderboard::Error> for ErrorKind {
    fn from(value: aoc_leaderboard::Error) -> Self {
        aoc_leaderboard::ErrorKind::from(value).into()
    }
}

impl From<&aoc_leaderboard::Error> for ErrorKind {
    fn from(value: &aoc_leaderboard::Error) -> Self {
        aoc_leaderboard::ErrorKind::from(value).into()
    }
}

/// Error type used for errors related to [`Storage`].
///
/// [`Storage`]: crate::leaderbot::Storage
#[derive(Debug, thiserror::Error, EnumDiscriminants, EnumIs)]
#[non_exhaustive]
#[strum_discriminants(
    name(StorageErrorKind),
    derive(Serialize, Deserialize, EnumIs),
    non_exhaustive
)]
pub enum StorageError {
    /// Error while trying to load previous leaderboard data.
    #[error("failed to load previous leaderboard data: {0}")]
    LoadPrevious(anyhow::Error),

    /// Error while trying to save new leaderboard data.
    #[error("failed to save leaderboard data: {0}")]
    SaveSuccess(anyhow::Error),

    /// Error while trying to save previous error.
    #[error("failed to save previous error: {0}")]
    SaveError(anyhow::Error),
}

impl StorageError {
    /// Returns `true` if the enum is [`StorageError::LoadPrevious`] and the internal
    /// [`anyhow::Error`] matches the given predicate.
    pub fn is_load_previous_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&anyhow::Error) -> bool,
    {
        match self {
            Self::LoadPrevious(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`StorageError::SaveSuccess`] and the internal
    /// [`anyhow::Error`] matches the given predicate.
    pub fn is_save_success_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&anyhow::Error) -> bool,
    {
        match self {
            Self::SaveSuccess(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`StorageError::SaveError`] and the internal
    /// [`anyhow::Error`] matches the given predicate.
    pub fn is_save_error_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&anyhow::Error) -> bool,
    {
        match self {
            Self::SaveError(source) => predicate(source),
            _ => false,
        }
    }
}

impl PartialEq<StorageErrorKind> for StorageError {
    fn eq(&self, other: &StorageErrorKind) -> bool {
        StorageErrorKind::from(self) == *other
    }
}

impl PartialEq<StorageError> for StorageErrorKind {
    fn eq(&self, other: &StorageError) -> bool {
        *self == Self::from(other)
    }
}

impl PartialEq<StorageErrorKind> for ErrorKind {
    fn eq(&self, other: &StorageErrorKind) -> bool {
        *self == ErrorKind::Storage(*other)
    }
}

impl PartialEq<ErrorKind> for StorageErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        ErrorKind::Storage(*self) == *other
    }
}

impl PartialEq<StorageErrorKind> for Error {
    fn eq(&self, other: &StorageErrorKind) -> bool {
        ErrorKind::from(self) == *other
    }
}

impl PartialEq<Error> for StorageErrorKind {
    fn eq(&self, other: &Error) -> bool {
        *self == ErrorKind::from(other)
    }
}

impl From<StorageErrorKind> for ErrorKind {
    fn from(value: StorageErrorKind) -> Self {
        ErrorKind::Storage(value)
    }
}

impl From<&StorageErrorKind> for ErrorKind {
    fn from(value: &StorageErrorKind) -> Self {
        (*value).into()
    }
}

impl From<StorageError> for ErrorKind {
    fn from(value: StorageError) -> Self {
        StorageErrorKind::from(value).into()
    }
}

impl From<&StorageError> for ErrorKind {
    fn from(value: &StorageError) -> Self {
        StorageErrorKind::from(value).into()
    }
}

/// Error type used for errors related to [`Reporter`].
///
/// [`Reporter`]: crate::leaderbot::Reporter
#[derive(Debug, thiserror::Error, EnumDiscriminants, EnumIs)]
#[non_exhaustive]
#[strum_discriminants(
    name(ReporterErrorKind),
    derive(Serialize, Deserialize, EnumIs),
    non_exhaustive
)]
pub enum ReporterError {
    /// Error while trying to report changes detected in leaderboard data.
    #[error("failed to report changes to leaderboard: {0}")]
    ReportChanges(anyhow::Error),

    /// Error while trying to report the first bot run.
    #[error("failed to report first run: {0}")]
    ReportFirstRun(anyhow::Error),
}

impl ReporterError {
    /// Returns `true` if the enum is [`ReporterError::ReportChanges`] and the internal
    /// [`anyhow::Error`] matches the given predicate.
    pub fn is_report_changes_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&anyhow::Error) -> bool,
    {
        match self {
            Self::ReportChanges(source) => predicate(source),
            _ => false,
        }
    }

    /// Returns `true` if the enum is [`ReporterError::ReportFirstRun`] and the internal
    /// [`anyhow::Error`] matches the given predicate.
    pub fn is_report_first_run_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&anyhow::Error) -> bool,
    {
        match self {
            Self::ReportFirstRun(source) => predicate(source),
            _ => false,
        }
    }
}

impl PartialEq<ReporterErrorKind> for ReporterError {
    fn eq(&self, other: &ReporterErrorKind) -> bool {
        ReporterErrorKind::from(self) == *other
    }
}

impl PartialEq<ReporterError> for ReporterErrorKind {
    fn eq(&self, other: &ReporterError) -> bool {
        *self == Self::from(other)
    }
}

impl PartialEq<ReporterErrorKind> for ErrorKind {
    fn eq(&self, other: &ReporterErrorKind) -> bool {
        *self == Self::Reporter(*other)
    }
}

impl PartialEq<ErrorKind> for ReporterErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        ErrorKind::Reporter(*self) == *other
    }
}

impl PartialEq<ReporterErrorKind> for Error {
    fn eq(&self, other: &ReporterErrorKind) -> bool {
        *self == ErrorKind::Reporter(*other)
    }
}

impl PartialEq<Error> for ReporterErrorKind {
    fn eq(&self, other: &Error) -> bool {
        ErrorKind::Reporter(*self) == *other
    }
}

impl From<ReporterErrorKind> for ErrorKind {
    fn from(value: ReporterErrorKind) -> Self {
        ErrorKind::Reporter(value)
    }
}

impl From<&ReporterErrorKind> for ErrorKind {
    fn from(value: &ReporterErrorKind) -> Self {
        (*value).into()
    }
}

impl From<ReporterError> for ErrorKind {
    fn from(value: ReporterError) -> Self {
        ReporterErrorKind::from(value).into()
    }
}

impl From<&ReporterError> for ErrorKind {
    fn from(value: &ReporterError) -> Self {
        ReporterErrorKind::from(value).into()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use anyhow::anyhow;
    use rstest::rstest;

    use super::*;

    fn missing_field_error() -> Error {
        Error::MissingField { target: "SomeType", field: "some_field" }
    }

    fn env_error() -> Error {
        Error::Env { var_name: "SOME_VAR".into(), source: EnvVarError::NotPresent }
    }

    fn leaderboard_error() -> Error {
        Error::Leaderboard(aoc_leaderboard::Error::NoAccess)
    }

    fn storage_error() -> Error {
        Error::Storage(StorageError::LoadPrevious(anyhow!("error")))
    }

    fn reporter_error() -> Error {
        Error::Reporter(ReporterError::ReportChanges(anyhow!("error")))
    }

    fn test_error_with_message() -> Error {
        Error::TestErrorWithMessage("error".into())
    }

    mod partial_eq_error_and_error_kind {
        use super::*;

        #[rstest]
        #[case::missing_field(missing_field_error(), ErrorKind::MissingField)]
        #[case::env(env_error(), ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::leaderboard(
            leaderboard_error(),
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
        )]
        #[case::storage(storage_error(), ErrorKind::Storage(StorageErrorKind::LoadPrevious))]
        #[case::reporter(reporter_error(), ErrorKind::Reporter(ReporterErrorKind::ReportChanges))]
        #[case::test_load_previous(Error::TestLoadPreviousError, ErrorKind::TestLoadPreviousError)]
        #[case::test_save_updated(Error::TestSaveUpdatedError, ErrorKind::TestSaveUpdatedError)]
        #[case::test_save_base(Error::TestSaveBaseError, ErrorKind::TestSaveBaseError)]
        #[case::test_save_error(Error::TestSaveErrorError, ErrorKind::TestSaveErrorError)]
        #[case::test_report_changes(
            Error::TestReportChangesError,
            ErrorKind::TestReportChangesError
        )]
        #[case::test_report_first_run(
            Error::TestReportFirstRunError,
            ErrorKind::TestReportFirstRunError
        )]
        #[case::test_error_with_message(test_error_with_message(), ErrorKind::TestErrorWithMessage)]
        fn for_variant(#[case] error: Error, #[case] error_kind: ErrorKind) {
            // This tests `PartialEq<ErrorKind> for Error`
            assert_eq!(error, error_kind);

            // This tests `PartialEq<Error> for ErrorKind`
            assert_eq!(error_kind, error);
        }
    }

    mod error_kind {
        use super::*;

        mod from_error_for_error_kind {
            use super::*;

            #[rstest]
            #[case::missing_field(missing_field_error(), ErrorKind::MissingField)]
            #[case::env(env_error(), ErrorKind::Env(EnvVarErrorKind::NotPresent))]
            #[case::leaderboard(
                leaderboard_error(),
                ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
            )]
            #[case::storage(storage_error(), ErrorKind::Storage(StorageErrorKind::LoadPrevious))]
            #[case::reporter(
                reporter_error(),
                ErrorKind::Reporter(ReporterErrorKind::ReportChanges)
            )]
            #[case::test_load_previous(
                Error::TestLoadPreviousError,
                ErrorKind::TestLoadPreviousError
            )]
            #[case::test_save_updated(Error::TestSaveUpdatedError, ErrorKind::TestSaveUpdatedError)]
            #[case::test_save_base(Error::TestSaveBaseError, ErrorKind::TestSaveBaseError)]
            #[case::test_save_error(Error::TestSaveErrorError, ErrorKind::TestSaveErrorError)]
            #[case::test_report_changes(
                Error::TestReportChangesError,
                ErrorKind::TestReportChangesError
            )]
            #[case::test_report_first_run(
                Error::TestReportFirstRunError,
                ErrorKind::TestReportFirstRunError
            )]
            #[case::test_error_with_message(
                test_error_with_message(),
                ErrorKind::TestErrorWithMessage
            )]
            fn for_variant(#[case] error: Error, #[case] expected_error_kind: ErrorKind) {
                let actual_error_kind: ErrorKind = error.into();
                assert_eq!(expected_error_kind, actual_error_kind);
            }
        }

        mod from_error_ref_for_error_kind {
            use super::*;

            #[rstest]
            #[case::missing_field(&missing_field_error(), ErrorKind::MissingField)]
            #[case::env(&env_error(), ErrorKind::Env(EnvVarErrorKind::NotPresent))]
            #[case::leaderboard(
                &leaderboard_error(),
                ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
            )]
            #[case::storage(&storage_error(), ErrorKind::Storage(StorageErrorKind::LoadPrevious))]
            #[case::reporter(
                &reporter_error(),
                ErrorKind::Reporter(ReporterErrorKind::ReportChanges)
            )]
            #[case::test_load_previous(
                &Error::TestLoadPreviousError,
                ErrorKind::TestLoadPreviousError
            )]
            #[case::test_save_updated(
                &Error::TestSaveUpdatedError,
                ErrorKind::TestSaveUpdatedError
            )]
            #[case::test_save_base(&Error::TestSaveBaseError, ErrorKind::TestSaveBaseError)]
            #[case::test_save_error(&Error::TestSaveErrorError, ErrorKind::TestSaveErrorError)]
            #[case::test_report_changes(
                &Error::TestReportChangesError,
                ErrorKind::TestReportChangesError
            )]
            #[case::test_report_first_run(
                &Error::TestReportFirstRunError,
                ErrorKind::TestReportFirstRunError
            )]
            #[case::test_error_with_message(&test_error_with_message(), ErrorKind::TestErrorWithMessage)]
            fn for_variant(#[case] error: &Error, #[case] expected_error_kind: ErrorKind) {
                let actual_error_kind: ErrorKind = error.into();
                assert_eq!(expected_error_kind, actual_error_kind);
            }
        }
    }
}
