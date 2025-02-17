//! [AWS Lambda] function implementation for [`aoc_leaderbot`].
//!
//! [AWS Lambda]: https://aws.amazon.com/lambda/
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

use std::fmt::Debug;

use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::DynamoDbStorage;
use aoc_leaderbot_lib::leaderbot::config::env::get_env_config;
use aoc_leaderbot_lib::leaderbot::config::mem::MemoryConfig;
use aoc_leaderbot_lib::leaderbot::{run_bot, Config};
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
    LeaderboardSortOrder, SlackWebhookReporter,
};
use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

/// Struct used to deserialize the incoming message passed
/// to our [AWS Lambda] function.
///
/// Allows caller to override fields of the [`Config`] and
/// other related parameters.
///
/// [AWS Lambda]: https://aws.amazon.com/lambda/
#[derive(Debug, Default, Clone, Deserialize)]
pub struct IncomingMessage {
    /// Year of leaderboard to monitor.
    ///
    /// If set, overrides [`Config::year`].
    #[serde(default)]
    pub year: Option<i32>,

    /// ID of leaderboard to monitor.
    ///
    /// If set, overrides [`Config::leaderboard_id`].
    #[serde(default)]
    pub leaderboard_id: Option<u64>,

    /// Advent of Code session token.
    ///
    /// If set, overrides [`Config::aoc_session`].
    #[serde(default)]
    pub aoc_session: Option<String>,

    /// AWS DynamoDB storage-specific input parameters.
    #[serde(flatten)]
    pub dynamodb_storage_input: IncomingDynamoDbStorageInput,

    /// Slack webhook reporter-specific input parameters.
    #[serde(flatten)]
    pub slack_webhook_reporter_input: IncomingSlackWebhookReporterInput,
}

/// AWS DynamoDB storage-specific part of the lambda's [`IncomingMessage`].
///
/// Allows caller to override the storage's table name.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct IncomingDynamoDbStorageInput {
    /// Name of DynamoDB table to use to store leaderboard data.
    pub table_name: Option<String>,
}

/// Slack webhook reporter-specific part of the lambda's [`IncomingMessage`].
///
/// Allows caller to override fields in the [`SlackWebhookReporter`].
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct IncomingSlackWebhookReporterInput {
    /// Slack webhook URL where to report changes.
    ///
    /// If set, overrides [`SlackWebhookReporter::webhook_url`].
    pub webhook_url: Option<String>,

    /// Slack channel where to report changes.
    ///
    /// If set, overrides [`SlackWebhookReporter::channel`].
    pub channel: Option<String>,

    /// Username to use when posting to Slack.
    ///
    /// If set, overrides [`SlackWebhookReporter::username`].
    pub username: Option<String>,

    /// URL of icon to use when posting to Slack.
    ///
    /// If set, overrides [`SlackWebhookReporter::icon_url`].
    pub icon_url: Option<String>,

    /// Leaderboard sort order to use when reporting changes.
    ///
    /// If set, overrides [`SlackWebhookReporter::sort_order`].
    pub sort_order: Option<LeaderboardSortOrder>,
}

/// Struct used to serialize the output of our [AWS Lambda] function.
///
/// [AWS Lambda]: https://aws.amazon.com/lambda/
#[derive(Debug, Clone, Serialize)]
pub struct OutgoingMessage {}

/// Prefix of environment variables used for the bot [`Config`] (see [`get_env_config`]).
pub const CONFIG_ENV_VAR_PREFIX: &str = "AOC_LEADERBOT_AWS_";

/// Default name of DynamoDB table used for the bot [`Storage`] (see [`DynamoDbStorage`]).
///
/// [`Storage`]: aoc_leaderbot_lib::leaderbot::Storage
pub const DEFAULT_DYNAMODB_TABLE_NAME: &str = "aoc_leaderbot";

/// [AWS Lambda] function handler that will be called to monitor an AoC leaderboard.
///
/// The handler will call the [`run_bot`] function using the following parameters:
///
/// - [`Config`] loaded from the environment (see [`get_env_config`]), possibly
///   overridden via the [input](IncomingMessage)
/// - [`DynamoDbStorage`]
/// - [`SlackWebhookReporter`]
///
/// [AWS Lambda]: https://aws.amazon.com/lambda/
#[instrument(ret, err)]
pub async fn bot_lambda_handler(
    event: LambdaEvent<IncomingMessage>,
) -> Result<OutgoingMessage, Error> {
    let input = event.payload;
    trace!(?input);

    let config = get_config(&input)?;
    let mut storage = get_storage(&input).await?;
    let mut reporter = get_reporter(&input)?;

    run_bot(&config, &mut storage, &mut reporter).await?;

    Ok(OutgoingMessage {})
}

#[instrument(err)]
fn get_config(input: &IncomingMessage) -> Result<MemoryConfig, Error> {
    let env_config = get_env_config(CONFIG_ENV_VAR_PREFIX)?;

    let year = input.year.unwrap_or_else(|| env_config.year());
    let leaderboard_id = input
        .leaderboard_id
        .unwrap_or_else(|| env_config.leaderboard_id());
    let aoc_session = input
        .aoc_session
        .clone()
        .unwrap_or_else(|| env_config.aoc_session());

    Ok(MemoryConfig::builder()
        .year(year)
        .leaderboard_id(leaderboard_id)
        .aoc_session(&aoc_session)
        .build()?)
}

#[instrument(err)]
async fn get_storage(input: &IncomingMessage) -> Result<DynamoDbStorage, Error> {
    let table_name = input
        .dynamodb_storage_input
        .table_name
        .clone()
        .unwrap_or_else(|| DEFAULT_DYNAMODB_TABLE_NAME.into());
    Ok(DynamoDbStorage::new(table_name).await)
}

#[instrument(err)]
fn get_reporter(input: &IncomingMessage) -> Result<SlackWebhookReporter, Error> {
    let mut builder = SlackWebhookReporter::builder();

    if let Some(webhook_url) = input.slack_webhook_reporter_input.webhook_url.clone() {
        builder.webhook_url(webhook_url);
    }
    if let Some(channel) = input.slack_webhook_reporter_input.channel.clone() {
        builder.channel(channel);
    }
    if let Some(username) = input.slack_webhook_reporter_input.username.clone() {
        builder.username(username);
    }
    if let Some(icon_url) = input.slack_webhook_reporter_input.icon_url.clone() {
        builder.icon_url(icon_url);
    }
    if let Some(sort_order) = input.slack_webhook_reporter_input.sort_order {
        builder.sort_order(sort_order);
    }

    Ok(builder.build()?)
}
