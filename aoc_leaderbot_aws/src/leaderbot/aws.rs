//! AWS-specific functionalities for [`aoc_leaderbot`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

#[cfg(feature = "dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
pub mod dynamo;
