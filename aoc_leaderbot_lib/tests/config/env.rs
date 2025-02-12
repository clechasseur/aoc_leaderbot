mod get_env_config {
    use std::env;

    use aoc_leaderbot_lib::error::EnvVarError;
    use aoc_leaderbot_lib::leaderbot::config::env::{
        get_env_config, ENV_CONFIG_AOC_SESSION_SUFFIX, ENV_CONFIG_LEADERBOARD_ID_SUFFIX,
        ENV_CONFIG_YEAR_SUFFIX,
    };
    use aoc_leaderbot_lib::leaderbot::Config;
    use aoc_leaderbot_lib::Error;
    use aoc_leaderbot_test_helpers::{AOC_SESSION, LEADERBOARD_ID, YEAR};
    use assert_matches::assert_matches;
    use chrono::{Datelike, Local};
    use uuid::Uuid;

    fn random_env_var_prefix() -> String {
        format!("test_{}_", Uuid::new_v4())
    }

    fn perform_valid_test(env_var_prefix: &str, set_year: bool) {
        let var_name = |name| format!("{env_var_prefix}{name}");

        if set_year {
            env::set_var(var_name(ENV_CONFIG_YEAR_SUFFIX), YEAR.to_string());
        }
        env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), LEADERBOARD_ID.to_string());
        env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), AOC_SESSION);

        let actual = get_env_config(env_var_prefix).unwrap();

        assert_eq!(actual.year(), if set_year { YEAR } else { Local::now().year() });
        assert_eq!(actual.leaderboard_id(), LEADERBOARD_ID);
        assert_eq!(actual.aoc_session(), AOC_SESSION);
    }

    #[test]
    fn with_year() {
        perform_valid_test(&random_env_var_prefix(), true);
    }

    #[test]
    fn without_year() {
        perform_valid_test(&random_env_var_prefix(), false);
    }

    mod missing_vars {
        use super::*;

        #[test]
        fn missing_leaderboard_id() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), AOC_SESSION);

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source: EnvVarError::NotPresent }) if actual_var_name == var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX));
        }

        #[test]
        fn missing_aoc_session() {
            let env_var_prefix = random_env_var_prefix();
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), LEADERBOARD_ID.to_string());

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
