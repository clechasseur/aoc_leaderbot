#[cfg(feature = "config-static")]
mod static_leaderbot_config {
    use aoc_leaderbot_lib::leaderbot::config::StaticLeaderbotConfig;
    use aoc_leaderbot_lib::leaderbot::LeaderbotConfig;

    #[test]
    fn new() {
        let actual = StaticLeaderbotConfig::new(2024, 12345, "aoc_session");

        assert_eq!(actual.year(), 2024);
        assert_eq!(actual.leaderboard_id(), 12345);
        assert_eq!(actual.aoc_session(), "aoc_session");
    }

    mod builder {
        use std::any::type_name;

        use aoc_leaderbot_lib::Error;
        use assert_matches::assert_matches;
        use chrono::{Datelike, Local};

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

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
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

            assert_matches!(actual, Err(Error::MissingField { target, field }) => {
                assert_eq!(target, type_name::<StaticLeaderbotConfig>());
                assert_eq!(field, "aoc_session");
            });
        }
    }
}

#[cfg(feature = "config-env")]
mod get_env_config {
    use std::env;

    use aoc_leaderbot_lib::error::EnvVarError;
    use aoc_leaderbot_lib::leaderbot::config::{
        get_env_config, ENV_CONFIG_AOC_SESSION_SUFFIX, ENV_CONFIG_LEADERBOARD_ID_SUFFIX,
        ENV_CONFIG_YEAR_SUFFIX,
    };
    use aoc_leaderbot_lib::leaderbot::LeaderbotConfig;
    use aoc_leaderbot_lib::Error;
    use assert_matches::assert_matches;
    use chrono::{Datelike, Local};
    use uuid::Uuid;

    fn random_env_var_prefix() -> String {
        format!("test_{}_", Uuid::new_v4())
    }

    fn perform_valid_test(env_var_prefix: &str, set_year: bool) {
        let var_name = |name| format!("{env_var_prefix}{name}");

        if set_year {
            env::set_var(var_name(ENV_CONFIG_YEAR_SUFFIX), "2024");
        }
        env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), "12345");
        env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), "aoc_session");

        let actual = get_env_config(env_var_prefix).unwrap();

        assert_eq!(actual.year(), if set_year { 2024 } else { Local::now().year() });
        assert_eq!(actual.leaderboard_id(), 12345);
        assert_eq!(actual.aoc_session(), "aoc_session");
    }

    mod empty_prefix {
        use super::*;

        #[test]
        fn with_year() {
            perform_valid_test("", true);
        }

        #[test]
        fn without_year() {
            perform_valid_test("", false);
        }
    }

    mod non_empty_prefix {
        use super::*;

        #[test]
        fn with_year() {
            perform_valid_test(&random_env_var_prefix(), true);
        }

        #[test]
        fn without_year() {
            perform_valid_test(&random_env_var_prefix(), false);
        }
    }

    mod missing_vars {
        use super::*;

        #[test]
        fn missing_leaderboard_id() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), "aoc_session");

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source: EnvVarError::NotPresent }) if actual_var_name == var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX));
        }

        #[test]
        fn missing_aoc_session() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), "12345");

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source: EnvVarError::NotPresent }) if actual_var_name == var_name(ENV_CONFIG_AOC_SESSION_SUFFIX));
        }
    }

    mod invalid_values {
        use super::*;

        #[test]
        fn invalid_year() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_YEAR_SUFFIX), "two-thousand-twenty-four");

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source }) => {
                assert_eq!(actual_var_name, var_name(ENV_CONFIG_YEAR_SUFFIX));
                assert_matches!(source, EnvVarError::IntExpected { actual, .. } if actual == "two-thousand-twenty-four");
            })
        }

        #[test]
        fn invalid_leaderboard_id() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), "one two three four five");

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source }) => {
                assert_eq!(actual_var_name, var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX));
                assert_matches!(source, EnvVarError::IntExpected { actual, .. } if actual == "one two three four five");
            })
        }
    }
}
