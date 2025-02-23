#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use aoc_leaderbot_aws_lambda_impl::leaderbot::DEFAULT_DYNAMODB_TABLE_NAME;
use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::DynamoDbStorage;
use aws_config::BehaviorVersion;
use clap::Parser;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let cli = Cli::parse();

    let storage = get_storage(&cli).await;
    storage.create_table().await?;

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn get_storage(cli: &Cli) -> DynamoDbStorage {
    if cli.test_endpoint_url.is_empty() {
        DynamoDbStorage::new(&cli.table_name).await
    } else {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region("ca-central-1")
            .test_credentials()
            .endpoint_url(&cli.test_endpoint_url)
            .load()
            .await;
        DynamoDbStorage::with_config(&config, &cli.table_name).await
    }
}

#[derive(Debug, Parser)]
#[command(version, about = "Prepare DynamoDB table for aoc_leaderbot", long_about = None)]
struct Cli {
    /// Name of DynamoDB table to use for leaderboard data
    #[arg(short, long, default_value_t = DEFAULT_DYNAMODB_TABLE_NAME.into())]
    pub table_name: String,

    /// Test endpoint URL. Used by tests only, not shown in help
    #[arg(long, hide = true, default_value_t)]
    pub test_endpoint_url: String,
}
