mod from_var_error_for_env_var_error {
    use std::env;

    use aoc_leaderbot_lib::error::EnvVarError;
    use assert_matches::assert_matches;

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
