//! Custom error type definition.

use std::env;
use std::ffi::OsString;
use std::num::ParseIntError;

/// Custom [`Result`](std::result::Result) type that defaults to this crate's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type used by this crate's API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Attempted to build an instance of a type via a builder, but
    /// a required field was missing.
    #[error("missing field {field} to build a {target}")]
    MissingField {
        /// Type of value that was supposed to be built.
        target: &'static str,

        /// Name of missing field.
        field: &'static str,
    },

    /// Error while getting the value of an environment variable.
    #[error("error fetching environment variable {var_name}: {source}")]
    Env {
        /// Name of environment variable.
        var_name: String,

        /// Error that occurred while trying to get environment variable's value.
        source: EnvVarError,
    },

    /// Error while fetching leaderboard data from the AoC website.
    #[error(transparent)]
    Leaderboard(#[from] aoc_leaderboard::Error),

    /// Error pertaining to an AWS service.
    #[cfg(feature = "aws-base")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-base")))]
    #[error(transparent)]
    Aws(#[from] AwsError),

    // The following errors are only used in tests, they will not be available to users.
    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestLoadPreviousError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestReportChangesError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestSaveUpdatedError,

    #[cfg(test)]
    #[doc(hidden)]
    #[error("test")]
    TestSaveBaseError,
}

/// A version of [`env::VarError`] with additional variants.
#[derive(Debug, Clone, thiserror::Error)]
pub enum EnvVarError {
    /// Environment variable is not present.
    ///
    /// Our equivalent of [`env::VarError::NotPresent`].
    #[error("variable not present in environment")]
    NotPresent,

    /// Environment variable contains invalid, non-Unicode characters.
    ///
    /// Our equivalent of [`env::VarError::NotUnicode`].
    #[error("variable contained invalid, non-Unicode characters")]
    NotUnicode(OsString),

    /// Environment variable was expected to contain an integer value but didn't.
    #[error("expected int value, found {actual}: {source}")]
    IntExpected {
        /// The actual content of the environment variable.
        actual: String,

        /// The error that occurred while parsing the environment variable's content.
        source: ParseIntError,
    },
}

impl From<env::VarError> for EnvVarError {
    fn from(value: env::VarError) -> Self {
        match value {
            env::VarError::NotPresent => EnvVarError::NotPresent,
            env::VarError::NotUnicode(value) => EnvVarError::NotUnicode(value),
        }
    }
}

/// Errors pertaining to AWS services.
#[cfg(feature = "aws-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-base")))]
#[derive(Debug, thiserror::Error)]
pub enum AwsError {
    /// DynamoDB error.
    #[cfg(feature = "aws-dynamo-base")]
    #[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
    #[error(transparent)]
    Dynamo(#[from] DynamoError),
}

/// Errors pertaining to the [AWS DynamoDB] service.
///
/// [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
#[cfg(feature = "aws-dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
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

#[cfg(feature = "aws-dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
impl From<DynamoError> for Error {
    fn from(value: DynamoError) -> Self {
        Self::Aws(value.into())
    }
}

/// Error pertaining to loading leaderboard data from DynamoDB.
#[cfg(feature = "aws-dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
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
#[cfg(feature = "aws-dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
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
#[cfg(feature = "aws-dynamo-base")]
#[cfg_attr(any(nightly_rustc, docsrs), doc(cfg(feature = "aws-dynamo-base")))]
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
