//! Library implementing AWS-specific functionalities for [`aoc_leaderbot`], a bot that can watch
//! an [Advent of Code] private leaderboard for changes and report them to various channels
//! like Slack.
//!
//! ## Trait implementations
//!
//! This library includes implementations of the traits found in [`aoc_leaderbot_lib`].
//!
//! ### [`DynamoDbStorage`]
//!
//! Required feature: `storage-dynamodb` (enabled by default)
//!
//! An implementation of the [`Storage`] trait that stores data in an [AWS DynamoDB] table.
//!
//! The only thing that the storage needs is the name of the table where to store data.
//! If that table does not yet exist, it's possible to create it via the [`create_table`].
//!
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot
//! [Advent of Code]: https://adventofcode.com/
//! [`DynamoDbStorage`]: leaderbot::storage::aws::dynamodb::DynamoDbStorage
//! [`Storage`]: aoc_leaderbot_lib::leaderbot::Storage
//! [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
//! [`create_table`]: leaderbot::storage::aws::dynamodb::DynamoDbStorage::create_table

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod error;
pub mod leaderbot;

pub use error::Error;
pub use error::Result;
