mod error {
    use anyhow::anyhow;
    use aoc_leaderbot_lib::error::{EnvVarError, ReporterError, StorageError};
    use aoc_leaderbot_lib::Error;

    mod is_something_and {
        use super::*;

        #[test]
        fn is_missing_field_and() {
            let predicate = |target, field| target == "SomeType" && field == "some_field";

            let error = Error::MissingField { target: "SomeType", field: "some_field" };
            assert!(error.is_missing_field_and(predicate));

            let error = Error::Env { var_name: "SOME_VAR".into(), source: EnvVarError::NotPresent };
            assert!(!error.is_missing_field_and(predicate));
        }

        #[test]
        fn is_env_and() {
            let predicate = |var_name, source: &EnvVarError| {
                var_name == "SOME_VAR" && matches!(source, EnvVarError::NotPresent)
            };

            let error = Error::Env { var_name: "SOME_VAR".into(), source: EnvVarError::NotPresent };
            assert!(error.is_env_and(predicate));

            let error = Error::MissingField { target: "SomeType", field: "some_field" };
            assert!(!error.is_env_and(predicate));
        }

        #[test]
        fn is_leaderboard_and() {
            let predicate = |leaderboard_err: &aoc_leaderboard::Error| {
                matches!(leaderboard_err, aoc_leaderboard::Error::NoAccess)
            };

            let error = Error::Leaderboard(aoc_leaderboard::Error::NoAccess);
            assert!(error.is_leaderboard_and(predicate));

            let error = Error::MissingField { target: "SomeType", field: "some_field" };
            assert!(!error.is_leaderboard_and(predicate));
        }

        #[test]
        fn is_storage_and() {
            let predicate =
                |storage_err: &StorageError| matches!(storage_err, StorageError::LoadPrevious(_));

            let error = Error::Storage(StorageError::LoadPrevious(anyhow!("error")));
            assert!(error.is_storage_and(predicate));

            let error = Error::MissingField { target: "SomeType", field: "some_field" };
            assert!(!error.is_storage_and(predicate));
        }

        #[test]
        fn is_reporter_and() {
            let predicate = |reporter_err: &ReporterError| {
                matches!(reporter_err, ReporterError::ReportChanges(_))
            };

            let error = Error::Reporter(ReporterError::ReportChanges(anyhow!("error")));
            assert!(error.is_reporter_and(predicate));

            let error = Error::MissingField { target: "SomeType", field: "some_field" };
            assert!(!error.is_reporter_and(predicate));
        }
    }
}

mod error_kind {
    use anyhow::anyhow;
    use aoc_leaderbot_lib::error::{
        EnvVarError, EnvVarErrorKind, ReporterError, ReporterErrorKind, StorageError,
        StorageErrorKind,
    };
    use aoc_leaderbot_lib::{Error, ErrorKind};
    use rstest::rstest;

    mod is_something_of_kind {
        use super::*;

        #[test]
        fn is_env_of_kind() {
            let error_kind = ErrorKind::Env(EnvVarErrorKind::NotPresent);
            assert!(error_kind.is_env_of_kind(EnvVarErrorKind::NotPresent));
            assert!(!error_kind.is_env_of_kind(EnvVarErrorKind::NotUnicode));
        }

        #[test]
        fn is_leaderboard_of_kind() {
            let error_kind = ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess);
            assert!(error_kind.is_leaderboard_of_kind(aoc_leaderboard::ErrorKind::NoAccess));
            assert!(!error_kind.is_leaderboard_of_kind(aoc_leaderboard::ErrorKind::HttpGet));
        }

        #[test]
        fn is_storage_of_kind() {
            let error_kind = ErrorKind::Storage(StorageErrorKind::Save);
            assert!(error_kind.is_storage_of_kind(StorageErrorKind::Save));
            assert!(!error_kind.is_storage_of_kind(StorageErrorKind::LoadPrevious));
        }

        #[test]
        fn is_reporter_of_kind() {
            let error_kind = ErrorKind::Reporter(ReporterErrorKind::ReportChanges);
            assert!(error_kind.is_reporter_of_kind(ReporterErrorKind::ReportChanges));
            // No other variant exists
        }
    }

    mod from_error_ref_for_error_kind {
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

        #[rstest]
        #[case::missing_field(missing_field_error(), ErrorKind::MissingField)]
        #[case::env(env_error(), ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::leaderboard(
            leaderboard_error(),
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
        )]
        #[case::storage(storage_error(), ErrorKind::Storage(StorageErrorKind::LoadPrevious))]
        #[case::reporter(reporter_error(), ErrorKind::Reporter(ReporterErrorKind::ReportChanges))]
        fn for_variant(#[case] error: Error, #[case] expected_error_kind: ErrorKind) {
            let actual_error_kind: ErrorKind = (&error).into();
            assert_eq!(expected_error_kind, actual_error_kind);
        }
    }
}

mod env_var_error {
    use std::env;

    use aoc_leaderbot_lib::error::EnvVarError;
    use assert_matches::assert_matches;

    mod from_var_error_for_env_var_error {
        use super::*;

        #[test_log::test]
        fn test_not_present() {
            let err = env::VarError::NotPresent;
            let actual: EnvVarError = err.into();

            assert_matches!(actual, EnvVarError::NotPresent);
        }

        #[test_log::test]
        fn test_not_unicode() {
            let err = env::VarError::NotUnicode("foo".into());
            let actual: EnvVarError = err.into();

            assert_matches!(actual, EnvVarError::NotUnicode(value) if value == "foo");
        }
    }
}
