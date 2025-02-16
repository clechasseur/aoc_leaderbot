# aoc_leaderbot_slack_lib

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderbot_slack_lib.svg)](https://crates.io/crates/aoc_leaderbot_slack_lib) [![downloads](https://img.shields.io/crates/d/aoc_leaderbot_slack_lib.svg)](https://crates.io/crates/aoc_leaderbot_slack_lib) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aoc_leaderbot_slack_lib) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Library implementing Slack-specific helpers for [`aoc_leaderbot`](https://github.com/clechasseur/aoc_leaderbot), a bot that can watch an [Advent of Code](https://adventofcode.com/) private leaderboard for changes and report them to various channels like Slack.

## Installing

Add `aoc_leaderbot_slack_lib` to your dependencies:

```toml
[dependencies]
aoc_leaderbot_slack_lib = "0.3.0"
```

or by running:

```bash
cargo add aoc_leaderbot_slack_lib
```

## Trait implementations

This library includes implementations of the traits found in [`aoc_leaderbot_lib`](https://crates.io/crates/aoc_leaderbot_lib).

### [`SlackWebhookReporter`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/struct.SlackWebhookReporter.html)

Required feature: `reporter-webhook` (enabled by default)

An implementation of the [`Reporter`](https://docs.rs/aoc_leaderbot_lib/latest/aoc_leaderbot_lib/leaderbot/trait.Reporter.html) trait that reports changes to the leaderboard to a Slack channel via a [Slack webhook](https://api.slack.com/messaging/webhooks).

The reporter has several configurable input properties.
Although most have default values, at least two must be specified explicitly:

* [`webhook_url`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/struct.SlackWebhookReporterBuilder.html#method.webhook_url): URL of the Slack webhook to use to report changes.
* [`channel`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/struct.SlackWebhookReporterBuilder.html#method.channel): Slack channel where to post message reporting changes.

There are other optional properties that can be specified.
The easiest way to create a reporter instance would be via the [`builder`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/struct.SlackWebhookReporterBuilder.html).
The buil

## Minimum Rust version

`aoc_leaderbot_slack_lib` currently builds on Rust 1.75 or newer.
