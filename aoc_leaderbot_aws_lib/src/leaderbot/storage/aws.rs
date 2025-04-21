//! Implementations of [`leaderbot::Storage`](aoc_leaderbot_lib::leaderbot::Storage) using AWS services.

#[cfg(feature = "storage-dynamodb")]
pub mod dynamodb;
