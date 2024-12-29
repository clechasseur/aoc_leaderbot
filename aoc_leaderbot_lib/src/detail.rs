use std::env;
use std::ffi::OsStr;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::error::EnvVarError;

pub fn int_env_var<T, K>(key: K) -> crate::Result<T>
where
    K: AsRef<OsStr>,
    T: FromStr<Err = ParseIntError>,
{
    let actual = env::var(key)?;
    match actual.parse::<T>() {
        Ok(val) => Ok(val),
        Err(source) => Err(EnvVarError::IntExpected { actual, source }.into()),
    }
}
