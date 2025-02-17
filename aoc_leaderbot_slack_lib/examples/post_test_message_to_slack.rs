//! Simple command-line tool to post a test message to Slack, reporting on
//! the current state of an [Advent of Code] [`Leaderboard`](Leaderboard) using a
//! [`SlackWebhookReporter`].
//!
//! [Advent of Code]: https://adventofcode.com/

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

use std::env;
use std::env::VarError;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderbot_lib::leaderbot::{Changes, Reporter};
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
    LeaderboardSortOrder, SlackWebhookReporter,
};
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::DEFAULT_USERNAME;
use chrono::{Datelike, Local};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let cli = Cli::parse_or_defaults()?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(cli.verbose.tracing_level_filter().into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let leaderboard = cli.get_leaderboard().await?;

    let mut reporter = cli.build_reporter()?;
    reporter
        .report_changes(
            leaderboard.year,
            leaderboard.owner_id,
            &leaderboard,
            &leaderboard,
            &Changes::default(),
        )
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Implements the `--verbose` and `--quiet` flags.
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Year of leaderboard to use.
    ///
    /// If not specified, will be fetched from the `AOC_LEADERBOARD_YEAR`
    /// environment variable. If the variable is not present, the current
    /// year will be used.
    #[arg(short, long)]
    pub year: Option<i32>,

    /// ID of leaderboard to use.
    ///
    /// If not specified, will be fetched from the `AOC_LEADERBOARD_ID`
    /// environment variable.
    #[arg(short = 'i', long = "id")]
    pub leaderboard_id: Option<u64>,

    /// Advent of Code session token.
    ///
    /// If not specified, will be fetched from the `AOC_SESSION`
    /// environment variable.
    #[arg(short = 't', long = "token")]
    pub aoc_session: Option<String>,

    /// URL of Slack webhook to post the test message to.
    ///
    /// If not specified, will be fetched from the `SLACK_WEBHOOK_URL`
    /// environment variable.
    #[arg(short, long)]
    pub webhook_url: Option<String>,

    /// Slack channel to post the test message to.
    ///
    /// If not specified, will be fetched from the `SLACK_CHANNEL`
    /// environment variable.
    #[arg(short, long)]
    pub channel: Option<String>,

    /// Username to use when posting to Slack.
    #[arg(short, long, default_value = DEFAULT_USERNAME)]
    pub username: String,

    /// URL of icon to use for the user posting to Slack.
    ///
    /// If not specified, the default icon will be used.
    #[arg(long)]
    pub icon_url: Option<String>,

    /// How to sort the leaderboard members in the message.
    #[arg(long, value_enum, default_value_t = LeaderboardSortOrder::Stars)]
    pub sort_order: LeaderboardSortOrder,
}

impl Cli {
    pub fn parse_or_defaults() -> anyhow::Result<Self> {
        let cli = Self::parse();

        let year = match cli.year {
            Some(year) => year,
            None => Self::optional_int_env_var("AOC_LEADERBOARD_YEAR")?
                .unwrap_or_else(|| Local::now().year()),
        };
        let leaderboard_id = match cli.leaderboard_id {
            Some(leaderboard_id) => leaderboard_id,
            None => Self::int_env_var("AOC_LEADERBOARD_ID")?,
        };
        let aoc_session = match cli.aoc_session {
            Some(aoc_session) => aoc_session,
            None => Self::env_var("AOC_SESSION")?,
        };

        Ok(Self {
            year: Some(year),
            leaderboard_id: Some(leaderboard_id),
            aoc_session: Some(aoc_session),
            ..cli
        })
    }

    async fn get_leaderboard(&self) -> anyhow::Result<Leaderboard> {
        Ok(Leaderboard::get(
            self.year.unwrap(),
            self.leaderboard_id.unwrap(),
            self.aoc_session.as_ref().unwrap(),
        )
        .await?)
    }

    pub fn build_reporter(&self) -> anyhow::Result<SlackWebhookReporter> {
        let mut builder = SlackWebhookReporter::builder();
        builder
            .username(self.username.clone())
            .sort_order(self.sort_order);

        if let Some(webhook_url) = &self.webhook_url {
            builder.webhook_url(webhook_url);
        }
        if let Some(channel) = &self.channel {
            builder.channel(channel);
        }
        if let Some(icon_url) = &self.icon_url {
            builder.icon_url(icon_url);
        }

        Ok(builder.build()?)
    }

    fn optional_env_var(var_name: &str) -> anyhow::Result<Option<String>> {
        match env::var(var_name) {
            Ok(value) => Ok(Some(value)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(value)) => Err(anyhow!(
                "environment variable {var_name} contains invalid unicode: {}",
                value.to_string_lossy()
            )),
        }
    }

    fn env_var(var_name: &str) -> anyhow::Result<String> {
        match Self::optional_env_var(var_name)? {
            Some(value) => Ok(value),
            None => Err(anyhow!("environment variable {var_name} is missing")),
        }
    }

    fn optional_int_env_var<T>(var_name: &str) -> anyhow::Result<Option<T>>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        match Self::optional_env_var(var_name)? {
            Some(value) => Ok(Some(value.parse()?)),
            None => Ok(None),
        }
    }

    fn int_env_var<T>(var_name: &str) -> anyhow::Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        Self::env_var(var_name).and_then(|value| {
            value
                .parse()
                .with_context(|| anyhow!("failed to parse environment variable {var_name}"))
        })
    }
}
