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
