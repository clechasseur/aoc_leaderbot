# aoc_leaderbot_lib

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderbot_lib.svg)](https://crates.io/crates/aoc_leaderbot_lib) [![downloads](https://img.shields.io/crates/d/aoc_leaderbot_lib.svg)](https://crates.io/crates/aoc_leaderbot_lib) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aoc_leaderbot_lib) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Library implementing the core functionalities of [`aoc_leaderbot`](https://github.com/clechasseur/aoc_leaderbot), a bot that can watch an [Advent of Code](https://adventofcode.com/) private leaderboard for changes and report them to various channels like Slack.

## Installing

Add `aoc_leaderbot_lib` to your dependencies:

```toml
[dependencies]
aoc_leaderbot_lib = "0.3.0"
```

or by running:

```bash
cargo add aoc_leaderbot_lib
```

## Usage

The bot's body is implemented via the [`run_bot`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/fn.run_bot.html) function.
This function will fetch the current version of the AoC leaderboard, then check if we had a previous version (from an earlier run).
If no previous version exists, the bot saves the current version and exists.
Otherwise, the bot compare the current leaderboard data with the previous one.
If there are new members or if existing members got new stars, it reports changes and saves the current version as the last one seen.

In order to function, the bot needs three things, which are passed using traits.

### [`Config`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Config.html)

This trait is used by the bot to fetch information about what AoC leaderboard to watch.
It is a read-only trait providing three pieces of information: the [leaderboard ID](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Config.html#tymethod.leaderboard_id), the [AoC session token](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Config.html#tymethod.aoc_session) and the [year](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Config.html#method.year).
The latter defaults to the current year.

### [`Storage`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Storage.html)

This trait abstracts the bot's storage facility.
It is used to load leaderboard data from a previous run and to save any new leaderboard data.

### [`Reporter`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Reporter.html)

This trait abstracts the bot's capability to report leaderboard changes when it finds some.
Its main purpose is to implement the [`report_changes`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Reporter.html#tymethod.report_changes) method to report changes to the user. This could be via a Slack post, a Discord message, etc.

The reporter can also be used to report any error occurring during bot execution (ex: expired AoC session token, etc.) via its [`report_error`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Reporter.html#tymethod.report_error) method.

## Concrete implementations

Although this library includes the bot's core function, it does not provide all possible implementations of the traits it needs for operations.
This library includes two implementations of `Config`, one implementation of `Storage` and no implementation of `Reporter`.
Users will thus need to implement a `Reporter` at a minimum.

For other trait implementations, you can look at related crates like [`aoc_leaderbot_slack_lib`](https://crates.io/crates/aoc_leaderbot_slack_lib).

### [`MemoryConfig`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/config/mem/struct.MemoryConfig.html)

Required feature: `config-mem` (enabled by default)

This implementation of `Config` simply stores its values in memory.
The most basic implementation, but it works.

### [`get_env_config`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/config/env/fn.get_env_config.html)

Required feature: `config-env` (enabled by default)

This function returns an opaque `Config` implementation fetching the parameters from environment variables.
This is possibly the most flexible way of providing the bot its config, which is why it is enabled by default.

### [`MemoryStorage`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/storage/mem/struct.MemoryStorage.html)

Required feature: `storage-mem`

This implementation of `Storage` simply stores its data in memory.
Although this means that it would technically lose its data upon program exit, the whole storage can be persisted using [`serde`](https://serde.rs/), which means it's a possibly-decent implementation.

## Minimum Rust version

`aoc_leaderbot_lib` currently builds on Rust 1.75 or newer.
