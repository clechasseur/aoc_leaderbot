//! [AWS Lambda] function implementation for [`aoc_leaderbot`].
//!
//! [AWS Lambda]: https://aws.amazon.com/lambda/
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

use std::borrow::Cow;
use std::fmt::Debug;

use aoc_leaderboard::aoc::LeaderboardCredentials;
use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::DynamoDbStorage;
use aoc_leaderbot_lib::leaderbot::config::env::get_env_config;
use aoc_leaderbot_lib::leaderbot::config::mem::MemoryConfig;
use aoc_leaderbot_lib::leaderbot::{BotOutput, Config, Reporter, run_bot_from};
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
    LeaderboardSortOrder, SlackWebhookReporter,
};
use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};
use veil::Redact;

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

    /// Advent of Code leaderboard credentials.
    ///
    /// If set, overrides [`Config::credentials`].
    #[serde(default)]
    pub credentials: Option<LeaderboardCredentials>,

    /// Set to `true` to do a test run.
    ///
    /// A test run will report changes even if there are none.
    #[serde(default)]
    pub test_run: bool,

    /// Base URL to use for the Advent of Code website.
    ///
    /// Should only be used for testing purposes.
    #[cfg(feature = "__testing")]
    #[doc(hidden)]
    #[serde(default)]
    pub aoc_base_url: Option<String>,

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

    /// Endpoint URL to use for local testing.
    ///
    /// Do not use this when lambda is deployed to AWS; all config
    /// values will be fetched from the lambda's environment.
    #[cfg(feature = "__testing")]
    #[doc(hidden)]
    pub test_endpoint_url: Option<String>,

    /// Region to use for local testing.
    ///
    /// Do not use this when lambda is deployed to AWS; all config
    /// values will be fetched from the lambda's environment.
    ///
    /// Ignored unless [`test_endpoint_url`](Self::test_endpoint_url) is set.
    #[cfg(feature = "__testing")]
    #[doc(hidden)]
    pub test_region: Option<String>,
}

/// Slack webhook reporter-specific part of the lambda's [`IncomingMessage`].
///
/// Allows caller to override fields in the [`SlackWebhookReporter`].
#[derive(Redact, Default, Clone, Deserialize)]
#[serde(default)]
pub struct IncomingSlackWebhookReporterInput {
    /// Slack webhook URL where to report changes.
    ///
    /// If set, overrides [`SlackWebhookReporter::webhook_url`].
    #[redact(partial)]
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
pub struct OutgoingMessage {
    /// [Output](BotOutput) of the bot's run.
    pub output: BotOutput,
}

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
/// [`run_bot`]: aoc_leaderbot_lib::leaderbot::run_bot
#[cfg_attr(not(coverage), tracing::instrument(ret, err))]
pub async fn bot_lambda_handler(
    event: LambdaEvent<IncomingMessage>,
) -> Result<OutgoingMessage, Error> {
    let input = event.payload;

    let config = get_config(&input)?;
    let mut storage = get_storage(&input).await;
    let mut reporter = get_reporter(&input)?;

    #[cfg(feature = "__testing")]
    let advent_of_code_base = input.aoc_base_url;
    #[cfg(not(feature = "__testing"))]
    let advent_of_code_base: Option<String> = None;

    trace!("Running bot (test run: {})", input.test_run);
    let output =
        run_bot_from(advent_of_code_base, &config, &mut storage, &mut reporter, input.test_run)
            .await?;

    if input.test_run {
        let previous_leaderboard = output
            .previous_leaderboard
            .as_ref()
            .unwrap_or(&output.leaderboard);
        let changes = output
            .changes
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default();

        info!("Test run: reporting changes");
        debug!(?previous_leaderboard, ?changes);
        reporter
            .report_changes(
                output.year,
                output.leaderboard_id,
                config.credentials().view_key(),
                previous_leaderboard,
                &output.leaderboard,
                &changes,
            )
            .await?;
    }

    Ok(OutgoingMessage { output })
}

#[cfg_attr(not(coverage), tracing::instrument(err))]
fn get_config(input: &IncomingMessage) -> Result<MemoryConfig, Error> {
    let (year, leaderboard_id, credentials) =
        match (input.year, input.leaderboard_id, input.credentials.clone()) {
            (Some(year), Some(leaderboard_id), Some(credentials)) => {
                (year, leaderboard_id, credentials)
            },
            (year, leaderboard_id, credentials) => {
                let env_config = get_env_config(CONFIG_ENV_VAR_PREFIX)?;
                (
                    year.unwrap_or_else(|| env_config.year()),
                    leaderboard_id.unwrap_or_else(|| env_config.leaderboard_id()),
                    credentials.unwrap_or_else(|| env_config.credentials()),
                )
            },
        };
    debug!(year, leaderboard_id, ?credentials);

    Ok(MemoryConfig::builder()
        .year(year)
        .leaderboard_id(leaderboard_id)
        .credentials(credentials)
        .build()
        .expect("all fields should have been specified"))
}

#[cfg_attr(not(coverage), tracing::instrument)]
async fn get_storage(input: &IncomingMessage) -> DynamoDbStorage {
    #[cfg(feature = "__testing")]
    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn internal_get_storage(input: &IncomingMessage, table_name: String) -> DynamoDbStorage {
        match input.dynamodb_storage_input.test_endpoint_url.as_ref() {
            Some(endpoint_url) => {
                let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                    .region(aws_config::Region::new(
                        input
                            .dynamodb_storage_input
                            .test_region
                            .as_ref()
                            .map(|region| Cow::Owned(region.clone()))
                            .unwrap_or_else(|| Cow::Borrowed("ca-central-1")),
                    ))
                    .endpoint_url(endpoint_url)
                    .test_credentials()
                    .load()
                    .await;
                DynamoDbStorage::with_config(&config, table_name).await
            },
            None => DynamoDbStorage::new(table_name).await,
        }
    }

    #[cfg(not(feature = "__testing"))]
    async fn internal_get_storage(_input: &IncomingMessage, table_name: String) -> DynamoDbStorage {
        DynamoDbStorage::new(table_name).await
    }

    let table_name = input
        .dynamodb_storage_input
        .table_name
        .clone()
        .unwrap_or_else(|| DEFAULT_DYNAMODB_TABLE_NAME.into());
    internal_get_storage(input, table_name).await
}

#[cfg_attr(not(coverage), tracing::instrument(err))]
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
