#![allow(dead_code)]

use std::env;
use std::ffi::OsStr;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::error::EnvVarError;

pub fn env_var<K>(key: K) -> crate::Result<String>
where
    K: AsRef<OsStr>,
{
    let key = key.as_ref();

    env::var(key).map_err(|err| crate::Error::Env {
        var_name: key.to_string_lossy().into(),
        source: err.into(),
    })
}

pub fn int_env_var<T, K>(key: K) -> crate::Result<T>
where
    K: AsRef<OsStr>,
    T: FromStr<Err = ParseIntError>,
{
    let key = key.as_ref();
    let actual = env_var(key)?;

    actual.parse().map_err(|source| crate::Error::Env {
        var_name: key.to_string_lossy().into(),
        source: EnvVarError::IntExpected { actual, source },
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use assert_matches::assert_matches;
    use rstest::{fixture, rstest};
    use serial_test::serial;
    use uuid::Uuid;

    use super::*;

    #[fixture]
    fn test_var_name() -> String {
        format!("test_{}", Uuid::new_v4())
    }

    mod env_var {
        use super::*;

        #[rstest]
        #[serial(env)]
        fn valid(test_var_name: String) {
            unsafe {
                env::set_var(&test_var_name, "foo");
            }

            let actual = env_var(&test_var_name);
            assert_matches!(actual, Ok(value) if value == "foo");
        }

        #[rstest]
        #[serial(env)]
        fn not_present(test_var_name: String) {
            unsafe {
                env::remove_var(&test_var_name); // Just in case 😉
            }

            let actual = env_var(&test_var_name);
            assert_matches!(actual, Err(crate::Error::Env { var_name, source }) => {
                assert_eq!(var_name, test_var_name);
                assert_matches!(source, EnvVarError::NotPresent);
            })
        }
    }

    mod int_env_var {
        use super::*;

        #[rstest]
        #[serial(env)]
        fn valid_int(test_var_name: String) {
            unsafe {
                env::set_var(&test_var_name, "42");
            }

            let actual = int_env_var::<i32, _>(&test_var_name);
            assert_matches!(actual, Ok(42));
        }

        #[rstest]
        #[serial(env)]
        fn not_present(test_var_name: String) {
            unsafe {
                env::remove_var(&test_var_name); // Just in case 😉
            }

            let actual = int_env_var::<i32, _>(&test_var_name);
            assert_matches!(actual, Err(crate::Error::Env { var_name, source }) => {
                assert_eq!(var_name, test_var_name);
                assert_matches!(source, EnvVarError::NotPresent);
            });
        }

        #[rstest]
        #[serial(env)]
        fn invalid_int(test_var_name: String) {
            unsafe {
                env::set_var(&test_var_name, "forty-two");
            }

            let actual = int_env_var::<i32, _>(&test_var_name);
            assert_matches!(actual, Err(crate::Error::Env { var_name, source }) => {
                assert_eq!(var_name, test_var_name);
                assert_matches!(source, EnvVarError::IntExpected { actual, .. } if actual == "forty-two");
            });
        }
    }
}
