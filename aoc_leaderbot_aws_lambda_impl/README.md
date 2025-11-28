# aoc_leaderbot_aws_lambda_impl

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderbot_aws_lambda_impl.svg)](https://crates.io/crates/aoc_leaderbot_aws_lambda_impl) [![MSRV](https://img.shields.io/crates/msrv/aoc_leaderbot_aws_lambda_impl)](https://github.com/clechasseur/aoc_leaderbot/tree/main/aoc_leaderbot_aws_lambda_impl) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Implementation of [`aoc_leaderbot`](https://github.com/clechasseur/aoc_leaderbot) that runs as an [AWS Lambda](https://aws.amazon.com/lambda/) function.
`aoc_leaderbot` is a bot that can watch an [Advent of Code](https://adventofcode.com/) private leaderboard for changes and report them to various channels.

## Installing

Installing `aoc_leaderbot_aws_lambda_impl` requires building the project, deploying it to your AWS account and setting up permissions and (optionally) scheduling its execution.
Before deploying, make sure your environment contains credentials to access your AWS account; for more information, see [this page](https://docs.aws.amazon.com/sdkref/latest/guide/access-login.html).

### A note about costs

This implementation of `aoc_leaderbot` uses serverless AWS services like [Lambda](https://aws.amazon.com/lambda/) and [DynamoDB](https://aws.amazon.com/dynamodb/) because at the time of this writing, those services were included in the [AWS Free Tier](https://aws.amazon.com/free/).
In theory, running the bot every 15 minutes continuously should be well below service limits of the Free Tier. Please note however that **this is not guaranteed** as AWS costs can be affected by many things and change change over time. Before deploying the bot, make sure you are aware of possible hosting costs.

### Prerequisites

- A clone of [this project](https://github.com/clechasseur/aoc_leaderbot)
- Rust 1.88 or newer (see [DEVELOPMENT](../DEVELOPMENT.md))
- [just](https://github.com/casey/just) (see [DEVELOPMENT](../DEVELOPMENT.md))
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

### Bot configuration

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

### Creating DynamoDB table

The bot stores leaderboard data in a [DynamoDB](https://aws.amazon.com/dynamodb/) table between runs.
Before running the bot for the first time, create the table by running:

```shell
just prepare-dynamo
```

This will make sure that the table is created with the proper hash and range key configuration.

### Building Lambda function

Build the Lambda function package so that it's ready for deployment by running:

```shell
just release=true build-lambda
```

Be patient as this can take a while the first time.

### Deploying Lambda function

Once the Lambda function package is built, it can be deployed by running:

```shell
just deploy-lambda
```

### Setting Up Permissions

Once the Lambda function is deployed, it needs to be given permission to read and write to the DynamoDB table.
This can be done by editing the [IAM role](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles.html) that has been created when the Lambda function was deployed.
There are different ways of doing this; to do it via the [AWS Management console](https://aws.amazon.com/console/):

1. Make sure to select the correct AWS region where the Lambda function was deployed
2. Using the Search or a shortcut, navigate to the Lambda component
3. Locate the bot's Lambda function and click to open it
4. Under the _Overview_ panel, select _Configuration_, then _Permissions_
5. Right under _Execution role_, there should be the name of the IAM role with a link to open the role; click on that link
6. Under _Permission policies_, click on _Add permissions_ and select _Create inline policy_
7. In the _Service_ selector, choose _DynamoDB_
8. Add the following permissions at a minimum:
   1. `GetItem`
   2. `PutItem`
   3. `DescribeTable`
9. Under _Resources_, click on _Add ARNs_ to add the ARN for the bot's DynamoDB table. The table name is `aoc_leaderbot`.
10. Click _Next_ to move to the next wizard page.
11. Under _Policy details_, give the policy a name.
12. Once ready, click on _Create policy_.

Please note that the AWS Management console might evolve over time and that the instructions above might become obsolete in the future; if in doubt, read the official documentation.
It is also possible to attach these permissions to the role programmatically or via the [AWS CLI](https://docs.aws.amazon.com/cli/).

### Running the bot on a schedule

Up to now, the bot should be functional - you can test it by invoking the bot's Lambda function with a test event (an empty payload should do if all the environment variables were correctly set in you `.env` file prior to deploying the bot).
If you want to run the bot on a schedule, you can add a trigger to the bot's Lambda function.
Again, there are various possible trigger types; one is [Amazon EventBridge](https://aws.amazon.com/eventbridge/).
Creating such a schedule is a bit outside the scope of this README, but you can peruse the [EventBridge documentation](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-what-is.html) for more information.

## Updating

If ever a new version of the bot is released and you want to update your bot's Lambda function, you can simply build and deploy it again using the instructions above.
This will deploy a new version of the Lambda function.
(If the new version has breaking changes, be sure to read the instructions on how to upgrade before deployment.)

## Contributing / Local development

For information about contributing to this project, see [CONTRIBUTING](../CONTRIBUTING.md).
For information regarding local development, see [DEVELOPMENT](../DEVELOPMENT.md).
