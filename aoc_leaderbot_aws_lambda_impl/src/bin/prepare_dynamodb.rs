use aoc_leaderbot_aws_lambda_impl::leaderbot::DEFAULT_DYNAMODB_TABLE_NAME;
use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::DynamoDbStorage;
use clap::Parser;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let cli = Cli::parse();

    let storage = DynamoDbStorage::new(cli.table_name).await;
    storage.create_table().await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about = "Prepare DynamoDB table for aoc_leaderbot", long_about = None)]
struct Cli {
    /// Name of DynamoDB table to use for leaderboard data
    #[arg(short, long, default_value_t = DEFAULT_DYNAMODB_TABLE_NAME.into())]
    pub table_name: String,
}
