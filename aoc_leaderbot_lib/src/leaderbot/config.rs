//! Implementations of [`leaderbot::Config`](crate::leaderbot::Config).

#[cfg(feature = "config-env")]
pub mod env;
#[cfg(feature = "config-mem")]
pub mod mem;
