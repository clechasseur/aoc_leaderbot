//! Implementations of [`leaderbot::Storage`](crate::leaderbot::Storage).

#[cfg(feature = "storage-mem")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "storage-mem")))]
pub mod mem;
