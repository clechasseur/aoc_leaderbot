# aoc_leaderbot

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

A bot that can monitor an [Advent of Code](https://adventofcode.com/) private leaderboard and report changes to services like Slack.
Written in Rust 🦀

## AWS Lambda-based implementation

Currently, the only implementation of the bot is designed to run as an [AWS Lambda](https://aws.amazon.com/lambda/) function.
For information on this implementation and how to deploy it, see the appropriate [README](./aoc_leaderbot_aws_lambda_impl/README.md).

## Contributing / Local development

For information about contributing to this project, see [CONTRIBUTING](./CONTRIBUTING.md).
For information regarding local development, see [DEVELOPMENT](./DEVELOPMENT.md).
