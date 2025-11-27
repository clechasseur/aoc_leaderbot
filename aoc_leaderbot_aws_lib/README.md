# aoc_leaderbot_aws_lib

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderbot_aws_lib.svg)](https://crates.io/crates/aoc_leaderbot_aws_lib) [![MSRV](https://img.shields.io/crates/msrv/aoc_leaderbot_aws_lib)](https://github.com/clechasseur/aoc_leaderbot/tree/main/aoc_leaderbot_aws_lib) [![downloads](https://img.shields.io/crates/d/aoc_leaderbot_aws_lib.svg)](https://crates.io/crates/aoc_leaderbot_aws_lib) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aoc_leaderbot_aws_lib) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Library implementing AWS-specific helpers for [`aoc_leaderbot`](https://github.com/clechasseur/aoc_leaderbot), a bot that can watch an [Advent of Code](https://adventofcode.com/) private leaderboard for changes and report them to various channels like Slack.

## Installing

Add `aoc_leaderbot_aws_lib` to your dependencies:

```toml
[dependencies]
aoc_leaderbot_aws_lib = "1.0.0"
```

or by running:

```shell
cargo add aoc_leaderbot_aws_lib
```

## Trait implementations

This library includes implementations of the traits found in [`aoc_leaderbot_lib`](https://crates.io/crates/aoc_leaderbot_lib).

### [`DynamoDbStorage`](https://docs.rs/aoc_leaderbot_aws_lib/latest/aoc_leaderbot_aws_lib/leaderbot/storage/aws/dynamodb/struct.DynamoDbStorage.html)

Required feature: `storage-dynamodb` (enabled by default)

An implementation of the [`Storage`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Storage.html) trait that stores data in an [AWS DynamoDB](https://aws.amazon.com/dynamodb/) table.

The only thing that the storage needs is the name of the table where to store data.
If that table does not yet exist, it's possible to create it via the [`create_table`](https://docs.rs/aoc_leaderbot_aws_lib/latest/aoc_leaderbot_aws_lib/leaderbot/storage/aws/dynamodb/struct.DynamoDbStorage.html#tymethod.create_table).

## Minimum Rust version

`aoc_leaderbot_aws_lib` currently builds on Rust 1.88 or newer.

## Contributing / Local development

For information about contributing to this project, see [CONTRIBUTING](../CONTRIBUTING.md).
For information regarding local development, see [DEVELOPMENT](../DEVELOPMENT.md).
