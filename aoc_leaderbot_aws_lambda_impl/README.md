# aoc_leaderbot_aws_lambda_impl

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderbot_aws_lambda_impl.svg)](https://crates.io/crates/aoc_leaderbot_aws_lambda_impl) [![MSRV](https://img.shields.io/crates/msrv/aoc_leaderbot_aws_lambda_impl)](https://github.com/clechasseur/aoc_leaderbot/tree/main/aoc_leaderbot_aws_lambda_impl) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Implementation of [`aoc_leaderbot`](https://github.com/clechasseur/aoc_leaderbot) that runs as an [AWS Lambda](https://aws.amazon.com/lambda/) function.
`aoc_leaderbot` is a bot that can watch an [Advent of Code](https://adventofcode.com/) private leaderboard for changes and report them to various channels.

## Installing

Installing `aoc_leaderbot_aws_lambda_impl` requires building the project, deploying it to your AWS account and setting up permissions and (optionally) scheduling its execution.
Before deploying, make sure your environment contains credentials to access your AWS account; for more information, see [this page](https://docs.aws.amazon.com/sdkref/latest/guide/access-login.html).

### Prerequisites

- A clone of [this project](https://github.com/clechasseur/aoc_leaderbot)
- Rust (see [DEVELOPMENT](../DEVELOPMENT.md))
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

### Bot Configuration

Create a file named [`.env`](../.env) at the project root and populate it with environment variables to configure the bot.

| Variable name                      | Content                                                                                                                                                                                                               | Required?      | Default value |
|------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------|---------------|
| `AOC_LEADERBOT_AWS_YEAR`           | Year to monitor                                                                                                                                                                                                       | ✓              | Current year  |
| `AOC_LEADERBOT_AWS_LEADERBOARD_ID` | ID of leaderboard to monitor <sup>1</sup>                                                                                                                                                                             | ✓              | -             |
| `AOC_LEADERBOT_AWS_VIEW_KEY`       | View key to access leaderboard's read-only link <sup>2</sup>                                                                                                                                                          | ✓ <sup>3</sup> | -             |
| `AOC_LEADERBOT_AWS_SESSION_COOKIE` | Cookie of Advent of Code session to access the leaderboard                                                                                                                                                            | ✓ <sup>3</sup> | -             |
| `SLACK_WEBHOOK_URL`                | URL of [Slack webhook](https://api.slack.com/messaging/webhooks) where to report changes                                                                                                                              | ✓              | -             |
| `SLACK_CHANNEL`                    | Slack channel where to report changes (without the `#`)                                                                                                                                                               | ✓              | -             |
| `SLACK_LEADERBOARD_SORT_ORDER`     | How to sort leaderboard members when reporting (see [`LeaderboardSortOrder`](https://docs.rs/aoc_leaderbot_slack_lib/latest/aoc_leaderbot_slack_lib/leaderbot/reporter/slack/webhook/enum.LeaderboardSortOrder.html)) |                | Stars         |

<sup>1</sup> : The leaderboard ID is the last part of the leaderboard's URL: `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}`.
<sup>2</sup> : If the leaderboard is accessible anonymously through a read-only link, the view key is passed as a query parameter: `https://adventofcode.com/{year}/leaderboard/private/view/{leaderboard_id}?view_key={view_key}`
<sup>3</sup> : Either the `VIEW_KEY` or the `SESSION_COOKIE` must be set. If both are set, the `VIEW_KEY` is used.

## Minimum Rust version

`aoc_leaderbot_aws_lambda_impl` currently builds on Rust 1.81 or newer.

## Contributing / Local development

For information about contributing to this project, see [CONTRIBUTING](../CONTRIBUTING.md).
For information regarding local development, see [DEVELOPMENT](../DEVELOPMENT.md).

_TODO remove the information below_

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

## Building

To build the project for production, run `cargo lambda build --release`. Remove the `--release` flag to build for development.

Read more about building your lambda function in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/build.html).

## Testing

You can run regular Rust unit tests with `cargo test`.

If you want to run integration tests locally, you can use the `cargo lambda watch` and `cargo lambda invoke` commands to do it.

First, run `cargo lambda watch` to start a local server. When you make changes to the code, the server will automatically restart.

Second, you'll need a way to pass the event data to the lambda function.

You can use the existent [event payloads](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events/src/fixtures) in the Rust Runtime repository if your lambda function is using one of the supported event types.

You can use those examples directly with the `--data-example` flag, where the value is the name of the file in the [lambda-events](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events/src/fixtures) repository without the `example_` prefix and the `.json` extension.

```bash
cargo lambda invoke --data-example apigw-request
```

For generic events, where you define the event data structure, you can create a JSON file with the data you want to test with. For example:

```json
{
    "command": "test"
}
```

Then, run `cargo lambda invoke --data-file ./data.json` to invoke the function with the data in `data.json`.


Read more about running the local server in [the Cargo Lambda documentation for the `watch` command](https://www.cargo-lambda.info/commands/watch.html).
Read more about invoking the function in [the Cargo Lambda documentation for the `invoke` command](https://www.cargo-lambda.info/commands/invoke.html).

## Deploying

To deploy the project, run `cargo lambda deploy`. This will create an IAM role and a Lambda function in your AWS account.

Read more about deploying your lambda function in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/deploy.html).
