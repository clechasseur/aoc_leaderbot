//! Custom error type definition.

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// DynamoDB error.
    #[cfg(feature = "dynamo-base")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
    #[error(transparent)]
    Dynamo(#[from] DynamoError),
}

/// Errors pertaining to the [AWS DynamoDB] service.
///
/// [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
#[cfg(feature = "dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
#[derive(Debug, thiserror::Error)]
pub enum DynamoError {
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
        source: LoadPreviousDynamoError,
    },

    /// Error occurred while saving leaderboard in DynamoDB table.
    #[error("failed to save leaderboard with id {leaderboard_id} for year {year}: {source}")]
    SaveLeaderboard {
        /// ID of leaderboard to persist.
        leaderboard_id: u64,

        /// Year to persist.
        year: i32,

        /// The error that occurred while trying to save leaderboard.
        source: SaveDynamoError,
    },

    /// Error occurred while creating a table to store leaderboard data
    #[error("failed to create table {table_name}: {source}")]
    CreateTable {
        /// Name of table that was to be created.
        table_name: String,

        /// The error that occurred while trying to create the table.
        source: CreateDynamoTableError,
    },
}

/// Error pertaining to loading leaderboard data from DynamoDB.
#[cfg(feature = "dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
#[derive(Debug, thiserror::Error)]
pub enum LoadPreviousDynamoError {
    /// Error that occurred while trying to load previous leaderboard data from DynamoDB.
    #[error("error loading leaderboard data: {0}")]
    GetItem(
        #[from]
        aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::get_item::GetItemError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >,
    ),

    /// The leaderboard row was fetched without issue, but it did not contain the leaderboard data.
    #[error("leaderboard data not found")]
    MissingLeaderboardData,

    /// The leaderboard data was fetched, but it wasn't persisted in a string.
    #[error("leaderboard data should be a string")]
    InvalidLeaderboardDataType,

    /// Failed to parse leaderboard data.
    #[error("failed to parse leaderboard data: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// Error pertaining to saving leaderboard data in DynamoDB.
#[cfg(feature = "dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
#[derive(Debug, thiserror::Error)]
pub enum SaveDynamoError {
    /// Error that occurred while trying to save leaderboard data in DynamoDB.
    #[error("error saving leaderboard data: {0}")]
    PutItem(
        #[from]
        aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::put_item::PutItemError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >,
    ),

    /// Failed to serialize leaderboard data.
    #[error("failed to serialize leaderboard data: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// Error pertaining to creating a DynamoDB table to store leaderboard data.
#[cfg(feature = "dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "dynamo-base")))]
#[derive(Debug, thiserror::Error)]
pub enum CreateDynamoTableError {
    /// Error that occurred while trying to create DynamoDB table.
    #[error("error creating table: {0}")]
    CreateTable(
        #[from]
        aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::create_table::CreateTableError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >,
    ),

    /// Error that occurred while trying to wait for DynamoDB table to be created.
    #[error("error getting table description: {0}")]
    DescribeTable(
        #[from]
        aws_sdk_dynamodb::error::SdkError<
            aws_sdk_dynamodb::operation::describe_table::DescribeTableError,
            aws_sdk_dynamodb::config::http::HttpResponse,
        >,
    ),
}
