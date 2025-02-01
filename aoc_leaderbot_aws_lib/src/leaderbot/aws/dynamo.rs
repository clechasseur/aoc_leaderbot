//! AWS DynamoDB-specific functionalities for [`aoc_leaderbot`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

#[cfg(feature = "storage-dynamo")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "storage-dynamo")))]
pub mod storage;
