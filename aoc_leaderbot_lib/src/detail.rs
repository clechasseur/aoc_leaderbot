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
mod tests {
    use super::*;
    
    mod int_env_var {
        use assert_matches::assert_matches;
        use serial_test::serial;
        use super::*;
        
        const TEST_ENV_VAR: &str = "INT_ENV_VAR_TEST_ENV_VAR";
        
        #[test]
        #[serial(int_env_var)]
        fn valid_int() {
            env::set_var(TEST_ENV_VAR, "42");
            
            let actual = int_env_var::<i32, _>(TEST_ENV_VAR);
            assert_matches!(actual, Ok(42));
        }
        
        #[test]
        #[serial(int_env_var)]
        fn not_present() {
            env::remove_var(TEST_ENV_VAR);
            
            let actual = int_env_var::<i32, _>(TEST_ENV_VAR);
            assert_matches!(actual, Err(crate::Error::Env(EnvVarError::NotPresent)));
        }

        #[test]
        #[serial(int_env_var)]
        fn invalid_int() {
            env::set_var(TEST_ENV_VAR, "forty-two");

            let actual = int_env_var::<i32, _>(TEST_ENV_VAR);
            assert_matches!(actual, Err(crate::Error::Env(EnvVarError::IntExpected { actual, .. })) => {
                assert_eq!(actual, "forty-two");
            });
        }
    }
}
