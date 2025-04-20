#[cfg(feature = "http")]
mod error {
    use std::collections::HashMap;

    use rstest::{fixture, rstest};

    #[fixture]
    fn reqwest_builder_error() -> reqwest::Error {
        // There's no way to create a `reqwest::Error` outside the reqwest crate,
        // so we'll have to trigger an actual error.
        let map_with_non_string_keys: HashMap<_, _> = [(true, 42), (false, 23)].into();
        reqwest::Client::new()
            .get("/test")
            .json(&map_with_non_string_keys)
            .build()
            .unwrap_err()
    }

    #[rstest]
    fn test_is_http_get_and(reqwest_builder_error: reqwest::Error) {
        let error: aoc_leaderboard::Error = reqwest_builder_error.into();
        assert!(error.is_http_get_and(|err| err.is_builder()));
    }
}

mod error_kind {
    use aoc_leaderboard::{Error, ErrorKind};

    mod impl_partial_eq_error_kind_for_error {
        use super::*;

        #[test]
        fn test_eq() {
            let error = Error::NoAccess;
            let error_kind = ErrorKind::NoAccess;
            assert_eq!(error, error_kind);
        }
    }

    mod impl_partial_eq_error_for_error_kind {
        use super::*;

        #[test]
        fn test_eq() {
            let error = Error::NoAccess;
            let error_kind = ErrorKind::NoAccess;
            assert_eq!(error_kind, error);
        }
    }
}
