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

use anyhow::{Context, anyhow};
use aoc_leaderboard::aoc::{Leaderboard, LeaderboardCredentials};
use aoc_leaderbot_lib::leaderbot::{Changes, Reporter};
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::DEFAULT_USERNAME;
use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
    LeaderboardSortOrder, SlackWebhookReporter,
};
use chrono::{Datelike, Local};
use clap::{Args, Parser};
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
    if cli.first_run {
        reporter
            .report_first_run(leaderboard.year, leaderboard.owner_id, cli.view_key(), &leaderboard)
            .await?;
    } else {
        reporter
            .report_changes(
                leaderboard.year,
                leaderboard.owner_id,
                cli.view_key(),
                &leaderboard,
                &leaderboard,
                &Changes::default(),
            )
            .await?;
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about = "Send test AoC leaderbot message to Slack", long_about = None)]
struct Cli {
    /// Implements the `--verbose` and `--quiet` flags
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Year of leaderboard to use
    ///
    /// If not specified, will be fetched from the `AOC_LEADERBOARD_YEAR`
    /// environment variable. If the variable is not present, the current
    /// year will be used.
    #[arg(short, long)]
    pub year: Option<i32>,

    /// ID of leaderboard to use
    ///
    /// If not specified, will be fetched from the `AOC_LEADERBOARD_ID`
    /// environment variable.
    #[arg(short = 'i', long = "id")]
    pub leaderboard_id: Option<u64>,

    #[command(flatten)]
    pub credentials: Credentials,

    /// URL of Slack webhook to post the test message to
    ///
    /// If not specified, will be fetched from the `SLACK_WEBHOOK_URL`
    /// environment variable.
    #[arg(short, long)]
    pub webhook_url: Option<String>,

    /// Slack channel to post the test message to
    ///
    /// If not specified, will be fetched from the `SLACK_CHANNEL`
    /// environment variable.
    #[arg(short, long)]
    pub channel: Option<String>,

    /// Username to use when posting to Slack.
    #[arg(short, long, default_value = DEFAULT_USERNAME)]
    pub username: String,

    /// URL of icon to use for the user posting to Slack
    ///
    /// If not specified, the default icon will be used.
    #[arg(long)]
    pub icon_url: Option<String>,

    /// How to sort the leaderboard members in the message
    #[arg(long, value_enum, default_value_t = LeaderboardSortOrder::Stars)]
    pub sort_order: LeaderboardSortOrder,

    /// Simulate the first bot run
    #[arg(short, long)]
    pub first_run: bool,
}

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
struct Credentials {
    /// Advent of Code leaderboard view key
    ///
    /// If not specified, will be fetched from the `AOC_VIEW_KEY`
    /// environment variable.
    #[arg(short = 'k', long = "key")]
    view_key: Option<String>,

    /// Advent of Code session token
    ///
    /// If not specified, will be fetched from the `AOC_SESSION`
    /// environment variable (unless a `key` is found).
    #[arg(short = 't', long = "token", value_name = "SESSION_TOKEN")]
    session_cookie: Option<String>,
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
        let credentials = cli.credentials.or_defaults()?;

        Ok(Self { year: Some(year), leaderboard_id: Some(leaderboard_id), credentials, ..cli })
    }

    fn view_key(&self) -> Option<&str> {
        self.credentials.view_key.as_deref()
    }

    async fn get_leaderboard(&self) -> anyhow::Result<Leaderboard> {
        Ok(Leaderboard::get(
            self.year.unwrap(),
            self.leaderboard_id.unwrap(),
            &self.credentials.as_leaderboard_credentials()?,
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

    pub(crate) fn optional_env_var(var_name: &str) -> anyhow::Result<Option<String>> {
        match env::var(var_name) {
            Ok(value) => Ok(Some(value)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(value)) => Err(anyhow!(
                "environment variable {var_name} contains invalid unicode: {}",
                value.to_string_lossy()
            )),
        }
    }

    pub(crate) fn env_var(var_name: &str) -> anyhow::Result<String> {
        match Self::optional_env_var(var_name)? {
            Some(value) => Ok(value),
            None => Err(anyhow!("environment variable {var_name} is missing")),
        }
    }

    pub(crate) fn optional_int_env_var<T>(var_name: &str) -> anyhow::Result<Option<T>>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        match Self::optional_env_var(var_name)? {
            Some(value) => Ok(Some(value.parse()?)),
            None => Ok(None),
        }
    }

    pub(crate) fn int_env_var<T>(var_name: &str) -> anyhow::Result<T>
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

impl Credentials {
    pub fn or_defaults(self) -> anyhow::Result<Self> {
        if self.view_key.is_none() && self.session_cookie.is_none() {
            return match Cli::optional_env_var("AOC_VIEW_KEY")? {
                Some(view_key) => Ok(Self { view_key: Some(view_key), session_cookie: None }),
                None => {
                    Ok(Self { view_key: None, session_cookie: Some(Cli::env_var("AOC_SESSION")?) })
                },
            };
        }

        Ok(self)
    }

    pub fn as_leaderboard_credentials(&self) -> anyhow::Result<LeaderboardCredentials> {
        match (&self.view_key, &self.session_cookie) {
            (Some(key), _) => Ok(LeaderboardCredentials::ViewKey(key.clone())),
            (None, Some(cookie)) => Ok(LeaderboardCredentials::SessionCookie(cookie.clone())),
            (None, None) => {
                Err(anyhow!("either view_key or session_cookie should be set at this point"))
            },
        }
    }
}
