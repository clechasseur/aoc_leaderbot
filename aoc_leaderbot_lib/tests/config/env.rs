mod get_env_config {
    use std::env;

    use aoc_leaderboard::test_helpers::{TEST_AOC_SESSION, TEST_LEADERBOARD_ID, TEST_YEAR};
    use aoc_leaderbot_lib::error::EnvVarError;
    use aoc_leaderbot_lib::leaderbot::config::env::{
        get_env_config, ENV_CONFIG_AOC_SESSION_SUFFIX, ENV_CONFIG_LEADERBOARD_ID_SUFFIX,
        ENV_CONFIG_YEAR_SUFFIX,
    };
    use aoc_leaderbot_lib::leaderbot::Config;
    use aoc_leaderbot_lib::Error;
    use assert_matches::assert_matches;
    use chrono::{Datelike, Local};
    use rstest::{fixture, rstest};
    use uuid::Uuid;

    #[fixture]
    fn env_var_prefix() -> String {
        format!("test_{}_", Uuid::new_v4())
    }

    #[rstest]
    #[test_log::test]
    fn valid(env_var_prefix: String, #[values(false, true)] set_year: bool) {
        let var_name = |name| format!("{env_var_prefix}{name}");

        if set_year {
            env::set_var(var_name(ENV_CONFIG_YEAR_SUFFIX), TEST_YEAR.to_string());
        }
        env::set_var(var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX), TEST_LEADERBOARD_ID.to_string());
        env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), TEST_AOC_SESSION);

        let actual = get_env_config(env_var_prefix).unwrap();

        assert_eq!(actual.year(), if set_year { TEST_YEAR } else { Local::now().year() });
        assert_eq!(actual.leaderboard_id(), TEST_LEADERBOARD_ID);
        assert_eq!(actual.aoc_session(), TEST_AOC_SESSION);
    }

    mod missing_vars {
        use super::*;

        #[rstest]
        #[test_log::test]
        fn missing_leaderboard_id(env_var_prefix: String) {
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_AOC_SESSION_SUFFIX), TEST_AOC_SESSION);

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source: EnvVarError::NotPresent }) if actual_var_name == var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX));
        }

        #[rstest]
        #[test_log::test]
        fn missing_aoc_session(env_var_prefix: String) {
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(
                var_name(ENV_CONFIG_LEADERBOARD_ID_SUFFIX),
                TEST_LEADERBOARD_ID.to_string(),
            );

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source: EnvVarError::NotPresent }) if actual_var_name == var_name(ENV_CONFIG_AOC_SESSION_SUFFIX));
        }
    }

    mod invalid_values {
        use super::*;

        #[rstest]
        #[test_log::test]
        fn invalid_year(env_var_prefix: String) {
            let var_name = |name| format!("{env_var_prefix}{name}");

            env::set_var(var_name(ENV_CONFIG_YEAR_SUFFIX), "two-thousand-twenty-four");

            let actual = get_env_config(&env_var_prefix);
            assert_matches!(actual, Err(Error::Env { var_name: actual_var_name, source }) => {
                assert_eq!(actual_var_name, var_name(ENV_CONFIG_YEAR_SUFFIX));
                assert_matches!(source, EnvVarError::IntExpected { actual, .. } if actual == "two-thousand-twenty-four");
            })
        }

        #[rstest]
        #[test_log::test]
        fn invalid_leaderboard_id(env_var_prefix: String) {
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
