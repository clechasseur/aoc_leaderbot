use std::collections::HashMap;
use std::env;

use anyhow::anyhow;
use aoc_leaderbot_lib::Error;
use aoc_leaderbot_lib::error::{EnvVarError, ReporterError, StorageError};

fn not_unicode_env_var_error() -> EnvVarError {
    EnvVarError::NotUnicode("foo".into())
}

fn int_expected_env_var_error() -> EnvVarError {
    let actual = "fourty-two".to_string();
    let source = actual.parse::<i32>().unwrap_err();
    EnvVarError::IntExpected { actual, source }
}

fn not_unicode_var_error() -> env::VarError {
    env::VarError::NotUnicode("foo".into())
}

fn aoc_leaderboard_http_get_error() -> aoc_leaderboard::Error {
    // There's no way to create a reqwest::Error outside the reqwest crate, so we'll
    // have to trigger an actual error to test this.
    let map_with_non_string_keys: HashMap<_, _> = [(true, "hello"), (false, "world")].into();
    let client = reqwest::Client::new();
    let http_error = client
        .get("/test")
        .json(&map_with_non_string_keys)
        .build()
        .unwrap_err();
    aoc_leaderboard::Error::from(http_error)
}

fn load_previous_storage_error() -> StorageError {
    StorageError::LoadPrevious(anyhow!("error"))
}

fn save_success_storage_error() -> StorageError {
    StorageError::SaveSuccess(anyhow!("error"))
}

fn save_error_storage_error() -> StorageError {
    StorageError::SaveError(anyhow!("error"))
}

fn load_previous_error() -> Error {
    Error::Storage(load_previous_storage_error())
}

fn save_success_error() -> Error {
    Error::Storage(save_success_storage_error())
}

fn save_error_error() -> Error {
    Error::Storage(save_error_storage_error())
}

fn report_changes_reporter_error() -> ReporterError {
    ReporterError::ReportChanges(anyhow!("error"))
}

fn report_first_run_reporter_error() -> ReporterError {
    ReporterError::ReportFirstRun(anyhow!("error"))
}

fn report_changes_error() -> Error {
    Error::Reporter(report_changes_reporter_error())
}

fn report_first_run_error() -> Error {
    Error::Reporter(report_first_run_reporter_error())
}

mod error {
    use super::*;

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
            let predicate = |var_name: &str, source: &EnvVarError| {
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
    use aoc_leaderbot_lib::ErrorKind;
    use aoc_leaderbot_lib::error::{EnvVarErrorKind, ReporterErrorKind, StorageErrorKind};
    use rstest::rstest;

    use super::*;

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
            let error_kind = ErrorKind::Storage(StorageErrorKind::SaveSuccess);
            assert!(error_kind.is_storage_of_kind(StorageErrorKind::SaveSuccess));
            assert!(!error_kind.is_storage_of_kind(StorageErrorKind::LoadPrevious));
        }

        #[test]
        fn is_reporter_of_kind() {
            let error_kind = ErrorKind::Reporter(ReporterErrorKind::ReportChanges);
            assert!(error_kind.is_reporter_of_kind(ReporterErrorKind::ReportChanges));
            assert!(!error_kind.is_reporter_of_kind(ReporterErrorKind::ReportFirstRun));
        }
    }

    mod from_env_var_error_kind_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(
            EnvVarErrorKind::NotPresent,
            ErrorKind::Env(EnvVarErrorKind::NotPresent)
        )]
        #[case::not_unicode(
            EnvVarErrorKind::NotUnicode,
            ErrorKind::Env(EnvVarErrorKind::NotUnicode)
        )]
        #[case::int_expected(
            EnvVarErrorKind::IntExpected,
            ErrorKind::Env(EnvVarErrorKind::IntExpected)
        )]
        fn for_variant(#[case] env_var_error_kind: EnvVarErrorKind, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = env_var_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_env_var_error_kind_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(&EnvVarErrorKind::NotPresent, ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::not_unicode(&EnvVarErrorKind::NotUnicode, ErrorKind::Env(EnvVarErrorKind::NotUnicode))]
        #[case::int_expected(&EnvVarErrorKind::IntExpected, ErrorKind::Env(EnvVarErrorKind::IntExpected))]
        fn for_variant(
            #[case] env_var_error_kind: &EnvVarErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = env_var_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_env_var_error_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(EnvVarError::NotPresent, ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::not_unicode(
            not_unicode_env_var_error(),
            ErrorKind::Env(EnvVarErrorKind::NotUnicode)
        )]
        #[case::int_expected(
            int_expected_env_var_error(),
            ErrorKind::Env(EnvVarErrorKind::IntExpected)
        )]
        fn for_variant(#[case] env_var_error: EnvVarError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = env_var_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_env_var_error_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(&EnvVarError::NotPresent, ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::not_unicode(&not_unicode_env_var_error(), ErrorKind::Env(EnvVarErrorKind::NotUnicode))]
        #[case::int_expected(&int_expected_env_var_error(), ErrorKind::Env(EnvVarErrorKind::IntExpected))]
        fn for_variant(#[case] env_var_error: &EnvVarError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = env_var_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_var_error_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(env::VarError::NotPresent, ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::not_unicode(not_unicode_var_error(), ErrorKind::Env(EnvVarErrorKind::NotUnicode))]
        fn for_variant(#[case] var_error: env::VarError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = var_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_var_error_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(&env::VarError::NotPresent, ErrorKind::Env(EnvVarErrorKind::NotPresent))]
        #[case::not_unicode(&not_unicode_var_error(), ErrorKind::Env(EnvVarErrorKind::NotUnicode))]
        fn for_variant(#[case] var_error: &env::VarError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = var_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_aoc_leaderboard_error_kind_for_error_kind {
        use super::*;

        #[rstest]
        #[case::http_get(
            aoc_leaderboard::ErrorKind::HttpGet,
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::HttpGet)
        )]
        #[case::no_access(
            aoc_leaderboard::ErrorKind::NoAccess,
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
        )]
        fn for_variant(
            #[case] aoc_leaderboard_error_kind: aoc_leaderboard::ErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = aoc_leaderboard_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_aoc_leaderboard_error_kind_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::http_get(&aoc_leaderboard::ErrorKind::HttpGet, ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::HttpGet))]
        #[case::no_access(&aoc_leaderboard::ErrorKind::NoAccess, ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess))]
        fn for_variant(
            #[case] aoc_leaderboard_error_kind: &aoc_leaderboard::ErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = aoc_leaderboard_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_aoc_leaderboard_error_for_error_kind {
        use super::*;

        #[rstest]
        #[case::http_get(
            aoc_leaderboard_http_get_error(),
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::HttpGet)
        )]
        #[case::no_access(
            aoc_leaderboard::Error::NoAccess,
            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess)
        )]
        fn for_variant(
            #[case] aoc_leaderboard_error: aoc_leaderboard::Error,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = aoc_leaderboard_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_aoc_leaderboard_error_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::http_get(&aoc_leaderboard_http_get_error(), ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::HttpGet))]
        #[case::no_access(&aoc_leaderboard::Error::NoAccess, ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess))]
        fn for_variant(
            #[case] aoc_leaderboard_error: &aoc_leaderboard::Error,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = aoc_leaderboard_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_storage_error_kind_for_error_kind {
        use super::*;

        #[rstest]
        #[case::load_previous(
            StorageErrorKind::LoadPrevious,
            ErrorKind::Storage(StorageErrorKind::LoadPrevious)
        )]
        #[case::save_success(
            StorageErrorKind::SaveSuccess,
            ErrorKind::Storage(StorageErrorKind::SaveSuccess)
        )]
        #[case::save_error(
            StorageErrorKind::SaveError,
            ErrorKind::Storage(StorageErrorKind::SaveError)
        )]
        fn for_variant(
            #[case] storage_error_kind: StorageErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = storage_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_storage_error_kind_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::load_previous(
            &StorageErrorKind::LoadPrevious,
            ErrorKind::Storage(StorageErrorKind::LoadPrevious),
        )]
        #[case::save_success(
            &StorageErrorKind::SaveSuccess,
            ErrorKind::Storage(StorageErrorKind::SaveSuccess),
        )]
        #[case::save_error(
            &StorageErrorKind::SaveError,
            ErrorKind::Storage(StorageErrorKind::SaveError),
        )]
        fn for_variant(
            #[case] storage_error_kind: &StorageErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = storage_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_storage_error_for_error_kind {
        use super::*;

        #[rstest]
        #[case::load_previous(
            load_previous_storage_error(),
            ErrorKind::Storage(StorageErrorKind::LoadPrevious)
        )]
        #[case::save_success(
            save_success_storage_error(),
            ErrorKind::Storage(StorageErrorKind::SaveSuccess)
        )]
        #[case::save_error(
            save_error_storage_error(),
            ErrorKind::Storage(StorageErrorKind::SaveError)
        )]
        fn for_variant(#[case] storage_error: StorageError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = storage_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_storage_error_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::load_previous(
            &load_previous_storage_error(),
            ErrorKind::Storage(StorageErrorKind::LoadPrevious),
        )]
        #[case::save_success(
            &save_success_storage_error(),
            ErrorKind::Storage(StorageErrorKind::SaveSuccess),
        )]
        #[case::save_error(
            &save_error_storage_error(),
            ErrorKind::Storage(StorageErrorKind::SaveError),
        )]
        fn for_variant(#[case] storage_error: &StorageError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = storage_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_reporter_error_kind_for_error_kind {
        use super::*;

        #[rstest]
        #[case::report_changes(
            ReporterErrorKind::ReportChanges,
            ErrorKind::Reporter(ReporterErrorKind::ReportChanges)
        )]
        #[case::report_first_run(
            ReporterErrorKind::ReportFirstRun,
            ErrorKind::Reporter(ReporterErrorKind::ReportFirstRun)
        )]
        fn for_variant(
            #[case] reporter_error_kind: ReporterErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = reporter_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_reporter_error_kind_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::report_changes(
            &ReporterErrorKind::ReportChanges,
            ErrorKind::Reporter(ReporterErrorKind::ReportChanges),
        )]
        #[case::report_first_run(
            &ReporterErrorKind::ReportFirstRun,
            ErrorKind::Reporter(ReporterErrorKind::ReportFirstRun),
        )]
        fn for_variant(
            #[case] reporter_error_kind: &ReporterErrorKind,
            #[case] error_kind: ErrorKind,
        ) {
            let error_kind_from: ErrorKind = reporter_error_kind.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_reporter_error_for_error_kind {
        use super::*;

        #[rstest]
        #[case::report_changes(
            report_changes_reporter_error(),
            ErrorKind::Reporter(ReporterErrorKind::ReportChanges)
        )]
        #[case::report_first_run(
            report_first_run_reporter_error(),
            ErrorKind::Reporter(ReporterErrorKind::ReportFirstRun)
        )]
        fn for_variant(#[case] reporter_error: ReporterError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = reporter_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }

    mod from_reporter_error_ref_for_error_kind {
        use super::*;

        #[rstest]
        #[case::report_changes(
            &report_changes_reporter_error(),
            ErrorKind::Reporter(ReporterErrorKind::ReportChanges),
        )]
        #[case::report_first_run(
            &report_first_run_reporter_error(),
            ErrorKind::Reporter(ReporterErrorKind::ReportFirstRun),
        )]
        fn for_variant(#[case] reporter_error: &ReporterError, #[case] error_kind: ErrorKind) {
            let error_kind_from: ErrorKind = reporter_error.into();
            assert_eq!(error_kind, error_kind_from);
        }
    }
}

mod env_var_error {
    use std::env;
    use std::ffi::OsStr;
    use std::num::{IntErrorKind, ParseIntError};

    use assert_matches::assert_matches;
    use gratte::IntoDiscriminant;
    use rstest::rstest;

    use super::*;

    mod is_something_and {
        use super::*;

        #[test]
        fn is_not_unicode_and() {
            let predicate = |invalid_os_str: &OsStr| !invalid_os_str.is_empty();

            let error = not_unicode_env_var_error();
            assert!(error.is_not_unicode_and(predicate));

            let error = EnvVarError::NotPresent;
            assert!(!error.is_not_unicode_and(predicate));
        }

        #[test]
        fn is_int_expected_and() {
            let predicate = |actual: &str, source: &ParseIntError| {
                !actual.is_empty() && *source.kind() == IntErrorKind::InvalidDigit
            };

            let error = int_expected_env_var_error();
            assert!(error.is_int_expected_and(predicate));

            let error = EnvVarError::NotPresent;
            assert!(!error.is_int_expected_and(predicate));
        }
    }

    mod partial_eq {
        use super::*;

        #[rstest]
        #[case::not_present(env::VarError::NotPresent, EnvVarError::NotPresent)]
        #[case::not_unicode(not_unicode_var_error(), not_unicode_env_var_error())]
        fn for_variant(#[case] var_error: env::VarError, #[case] env_var_error: EnvVarError) {
            // This tests `PartialEq<EnvVarError> for env::VarError`
            assert_eq!(var_error, env_var_error);

            // This tests `PartialEq<env::VarError> for EnvVarError`
            assert_eq!(env_var_error, var_error);

            // This tests `PartialEq<EnvVarErrorKind> for EnvVarError`
            let env_var_error_kind = env_var_error.discriminant();
            assert_eq!(env_var_error, env_var_error_kind);

            // This tests `PartialEq<EnvVarError> for EnvVarErrorKind`
            assert_eq!(env_var_error_kind, env_var_error);

            // This tests `PartialEq<ErrorKind> for EnvVarErrorKind`
            let error = Error::Env { var_name: "SOME_VAR".into(), source: env_var_error };
            let error_kind = error.discriminant();
            assert_eq!(env_var_error_kind, error_kind);

            // This tests `PartialEq<EnvVarErrorKind> for ErrorKind`
            assert_eq!(error_kind, env_var_error_kind);

            // This tests `PartialEq<Error> for EnvVarErrorKind`
            assert_eq!(env_var_error_kind, error);

            // This tests `PartialEq<EnvVarErrorKind> for Error`
            assert_eq!(error, env_var_error_kind);

            // This tests `PartialEq<env::VarError> for EnvVarErrorKind`
            assert_eq!(env_var_error_kind, var_error);

            // This tests `PartialEq<EnvVarErrorKind> for env::VarError`
            assert_eq!(var_error, env_var_error_kind);
        }

        #[test]
        fn for_int_expected() {
            // `IntExpected` doesn't exist in env::VarError
            assert_ne!(env::VarError::NotPresent, int_expected_env_var_error());
        }
    }

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

mod env_var_error_kind {
    use std::env;

    use aoc_leaderbot_lib::error::EnvVarErrorKind;
    use rstest::rstest;

    use super::*;

    mod from_var_error_for_env_var_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(env::VarError::NotPresent, EnvVarErrorKind::NotPresent)]
        #[case::not_unicode(not_unicode_var_error(), EnvVarErrorKind::NotUnicode)]
        fn for_variant(
            #[case] var_error: env::VarError,
            #[case] env_var_error_kind: EnvVarErrorKind,
        ) {
            let env_var_error_kind_from: EnvVarErrorKind = var_error.into();
            assert_eq!(env_var_error_kind, env_var_error_kind_from);
        }
    }

    mod from_var_error_ref_for_env_var_error_kind {
        use super::*;

        #[rstest]
        #[case::not_present(&env::VarError::NotPresent, EnvVarErrorKind::NotPresent)]
        #[case::not_unicode(&not_unicode_var_error(), EnvVarErrorKind::NotUnicode)]
        fn for_variant(
            #[case] var_error: &env::VarError,
            #[case] env_var_error_kind: EnvVarErrorKind,
        ) {
            let env_var_error_kind_from: EnvVarErrorKind = var_error.into();
            assert_eq!(env_var_error_kind, env_var_error_kind_from);
        }
    }
}

mod storage_error {
    use super::*;

    mod is_something_and {
        use super::*;

        #[test]
        fn is_load_previous_and() {
            let predicate = |anyhow_err: &anyhow::Error| !format!("{anyhow_err:?}").is_empty();

            let error = StorageError::LoadPrevious(anyhow!("error"));
            assert!(error.is_load_previous_and(predicate));

            let error = StorageError::SaveSuccess(anyhow!("error"));
            assert!(!error.is_load_previous_and(predicate));
        }

        #[test]
        fn is_save_success_and() {
            let predicate = |anyhow_err: &anyhow::Error| !format!("{anyhow_err:?}").is_empty();

            let error = StorageError::SaveSuccess(anyhow!("error"));
            assert!(error.is_save_success_and(predicate));

            let error = StorageError::LoadPrevious(anyhow!("error"));
            assert!(!error.is_save_success_and(predicate));
        }

        #[test]
        fn is_save_error_and() {
            let predicate = |anyhow_err: &anyhow::Error| !format!("{anyhow_err:?}").is_empty();

            let error = StorageError::SaveError(anyhow!("error"));
            assert!(error.is_save_error_and(predicate));

            let error = StorageError::LoadPrevious(anyhow!("error"));
            assert!(!error.is_save_error_and(predicate));
        }
    }
}

mod storage_error_kind {
    use gratte::IntoDiscriminant;
    use rstest::rstest;

    use super::*;

    mod partial_eq {
        use super::*;

        #[rstest]
        #[case::load_previous(load_previous_storage_error(), load_previous_error())]
        #[case::save_success(save_success_storage_error(), save_success_error())]
        #[case::save_error(save_error_storage_error(), save_error_error())]
        fn for_variant(#[case] storage_error: StorageError, #[case] error: Error) {
            let storage_error_kind = storage_error.discriminant();
            let error_kind = error.discriminant();

            // This tests `PartialEq<StorageErrorKind> for StorageError`
            assert_eq!(storage_error, storage_error_kind);

            // This tests `PartialEq<StorageError> for StorageErrorKind`
            assert_eq!(storage_error_kind, storage_error);

            // This tests `PartialEq<ErrorKind> for StorageErrorKind`
            assert_eq!(storage_error_kind, error_kind);

            // This tests `PartialEq<StorageErrorKind> for ErrorKind`
            assert_eq!(error_kind, storage_error_kind);

            // This tests `PartialEq<StorageErrorKind> for Error`
            assert_eq!(error, storage_error_kind);

            // This tests `PartialEq<Error> for StorageErrorKind`
            assert_eq!(storage_error_kind, error);
        }
    }
}

mod reporter_error {
    use gratte::IntoDiscriminant;
    use rstest::rstest;

    use super::*;

    mod is_something_and {
        use super::*;

        #[test]
        fn is_report_changes_and() {
            let predicate = |anyhow_err: &anyhow::Error| !format!("{anyhow_err:?}").is_empty();

            let error = ReporterError::ReportChanges(anyhow!("error"));
            assert!(error.is_report_changes_and(predicate));

            let error = ReporterError::ReportFirstRun(anyhow!("error"));
            assert!(!error.is_report_changes_and(predicate));
        }

        #[test]
        fn is_report_first_run_and() {
            let predicate = |anyhow_err: &anyhow::Error| !format!("{anyhow_err:?}").is_empty();

            let error = ReporterError::ReportFirstRun(anyhow!("error"));
            assert!(error.is_report_first_run_and(predicate));

            let error = ReporterError::ReportChanges(anyhow!("error"));
            assert!(!error.is_report_first_run_and(predicate));
        }
    }

    mod partial_eq {
        use super::*;

        #[rstest]
        #[case::report_changes(report_changes_reporter_error(), report_changes_error())]
        #[case::report_first_run(report_first_run_reporter_error(), report_first_run_error())]
        fn for_variant(#[case] reporter_error: ReporterError, #[case] error: Error) {
            let reporter_error_kind = reporter_error.discriminant();
            let error_kind = error.discriminant();

            // This tests `PartialEq<ReporterErrorKind> for ReporterError`
            assert_eq!(reporter_error, reporter_error_kind);

            // This tests `PartialEq<ReporterError> for ReporterErrorKind`
            assert_eq!(reporter_error_kind, reporter_error);

            // This tests `PartialEq<ReporterErrorKind> for ErrorKind`
            assert_eq!(error_kind, reporter_error_kind);

            // This tests `PartialEq<ErrorKind> for ReporterErrorKind`
            assert_eq!(reporter_error_kind, error_kind);

            // This tests `PartialEq<ReporterErrorKind> for Error`
            assert_eq!(error, reporter_error_kind);

            // This tests `PartialEq<Error> for ReporterErrorKind`
            assert_eq!(reporter_error_kind, error);
        }
    }
}
