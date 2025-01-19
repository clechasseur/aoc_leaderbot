//! Implementations of [`leaderbot::Config`](crate::leaderbot::Config).

#[cfg(feature = "config-env")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-env")))]
pub mod env;
#[cfg(feature = "config-mem")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "config-mem")))]
pub mod mem;
