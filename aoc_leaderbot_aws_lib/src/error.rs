//! Custom error type definition.

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// DynamoDB error.
    #[cfg(feature = "dynamodb-base")]
    #[error(transparent)]
    Dynamo(#[from] DynamoDbError),
}

/// Errors pertaining to the [AWS DynamoDB] service.
///
/// [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
#[cfg(feature = "dynamodb-base")]
#[derive(Debug, thiserror::Error)]
pub enum DynamoDbError {
    /// Error occurred while loading previous leaderboard from DynamoDB table.
    #[error(
        "failed to load previous leaderboard with id {leaderboard_id} for year {year}: {source}"
    )]
    LoadPreviousLeaderboard {
        /// ID of requested leaderboard.
        leaderboard_id: u64,

        /// Requested year.
        year: i32,

        /// The error that occurred while trying to load previous leaderboard.
        source: LoadPreviousDynamoDbError,
    },

    /// Error occurred while saving leaderboard in DynamoDB table.
    #[error("failed to save leaderboard with id {leaderboard_id} for year {year}: {source}")]
    SaveLeaderboard {
        /// ID of leaderboard to persist.
        leaderboard_id: u64,

        /// Year to persist.
        year: i32,

        /// The error that occurred while trying to save leaderboard.
        source: SaveDynamoDbError,
    },

    /// Error occurred while saving last error information in DynamoDB table.
    #[error("failed to save last error information for leaderboard with id {leaderboard_id} for year {year}: {source}")]
    SaveLastError {
        /// ID of leaderboard to persist.
        leaderboard_id: u64,

        /// Year to persist.
        year: i32,

        /// The error that occurred while trying to save last error information.
        source: SaveDynamoDbError,
    },

    /// Error occurred while creating a table to store leaderboard data
    #[error("failed to create table {table_name}: {source}")]
    CreateTable {
        /// Name of table that was to be created.
        table_name: String,

        /// The error that occurred while trying to create the table.
        source: CreateDynamoDbTableError,
    },
}

/// Error pertaining to loading data from DynamoDB.
#[cfg(feature = "dynamodb-base")]
#[derive(Debug, thiserror::Error)]
pub enum LoadPreviousDynamoDbError {
    /// Error that occurred while trying to load previous leaderboard data from DynamoDB.
    #[error("error loading leaderboard data: {0}")]
    GetItem(
        #[from]
        Box<aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::get_item::GetItemError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >>,
    ),

    /// Failed to deserialize leaderboard data.
    #[error("failed to deserialize leaderboard data: {0}")]
    Deserialize(#[from] serde_dynamo::Error),
}

/// Error pertaining to saving data in DynamoDB.
#[cfg(feature = "dynamodb-base")]
#[derive(Debug, thiserror::Error)]
pub enum SaveDynamoDbError {
    /// Error that occurred while trying to save data in DynamoDB.
    #[error("error saving leaderboard data: {0}")]
    PutItem(
        #[from]
        Box<aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::put_item::PutItemError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >>,
    ),

    /// Error that occurred while trying to upsert data in DynamoDB.
    #[error("error upserting last error information: {0}")]
    UpdateItem(
        #[from]
        Box<aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::update_item::UpdateItemError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >>,
    ),

    /// Failed to serialize data to DynamoDB format.
    #[error("failed to serialize data for DynamoDB: {0}")]
    Serialize(#[from] serde_dynamo::Error),
}

/// Error pertaining to creating a DynamoDB table to store leaderboard data.
#[cfg(feature = "dynamodb-base")]
#[derive(Debug, thiserror::Error)]
pub enum CreateDynamoDbTableError {
    /// Error that occurred while trying to create DynamoDB table.
    #[error("error creating table: {0}")]
    CreateTable(
        #[from]
        Box<aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::create_table::CreateTableError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >>,
    ),

    /// Error that occurred while trying to wait for DynamoDB table to be created.
    #[error("error getting table description: {0}")]
    DescribeTable(
        #[from]
        Box<aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::describe_table::DescribeTableError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >>,
    ),
}
