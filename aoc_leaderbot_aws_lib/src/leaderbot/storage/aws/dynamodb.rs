//! [`leaderbot::Storage`](Storage) keeping data in an [AWS DynamoDB] table.
//!
//! [AWS DynamoDB]: https://aws.amazon.com/dynamodb/

#[cfg(feature = "__test_helpers")]
#[doc(hidden)]
pub mod test_helpers;

use std::time::Duration;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderbot_lib::leaderbot::Storage;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::operation::create_table::CreateTableOutput;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType,
    ScalarAttributeType, TableDescription, TableStatus,
};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use aoc_leaderbot_lib::ErrorKind;
use crate::error::DynamoDbError;

/// The hash key (aka partition key) used by [`DynamoDbStorage`].
///
/// Stores the `leaderboard_id`.
pub const HASH_KEY: &str = "leaderboard_id";

/// The range key used by [`DynamoDbStorage`].
///
/// Stores the `year`.
pub const RANGE_KEY: &str = "year";

/// The column storing leaderboard data in the [`DynamoDbStorage`].
pub const LEADERBOARD_DATA: &str = "leaderboard_data";

/// The column storing last error information in the [`DynamoDbStorage`].
pub const LAST_ERROR: &str = "last_error";

/// Newtype struct used to persist last error information into
/// a DynamoDB table. Used by [`DynamoDbStorage`].
///
/// Serializes transparently into an [`ErrorKind`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DynamoDbLastErrorInformation(ErrorKind);

/// Struct used to persist [`Leaderboard`] data into a DynamoDB
/// table. Used by [`DynamoDbStorage`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamoDbLeaderboardData {
    /// Leaderboard ID. Stored in the table's [`HASH_KEY`].
    pub leaderboard_id: u64,

    /// Year of leaderboard data. Stored in the table's [`RANGE_KEY`].
    pub year: i32,

    /// Structured leaderboard data. Stored in the [`LEADERBOARD_DATA`] column.
    #[serde(default)]
    pub leaderboard_data: Option<Leaderboard>,

    /// Information about last execution error, if any. Stored in the [`LAST_ERROR`] column.
    #[serde(default)]
    pub last_error: Option<DynamoDbLastErrorInformation>,
}

impl DynamoDbLeaderboardData {
    /// Creates a [`DynamoDbLeaderboardData`] to store the result of a successful bot run.
    pub fn for_success(year: i32, leaderboard_id: u64, leaderboard: Leaderboard) -> Self {
        Self {
            leaderboard_id,
            year,
            leaderboard_data: Some(leaderboard),
            last_error: None,
        }
    }
}

/// Bot storage that keeps data in an [AWS DynamoDB] table.
///
/// [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
#[derive(Debug, Clone)]
pub struct DynamoDbStorage {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl DynamoDbStorage {
    /// Creates a new DynamoDB bot storage.
    ///
    /// The only parameter required is the DynamoDB table name.
    /// AWS SDK config will be loaded from the environment.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new<T>(table_name: T) -> Self
    where
        T: Into<String>,
    {
        let config = aws_config::load_from_env().await;
        Self::with_config(&config, table_name).await
    }

    /// Creates a new DynamoDB bot storage using the provided AWS SDK config.
    pub async fn with_config<T>(config: &SdkConfig, table_name: T) -> Self
    where
        T: Into<String>,
    {
        Self { client: aws_sdk_dynamodb::Client::new(config), table_name: table_name.into() }
    }

    /// Creates a DynamoDB table suitable for storing leaderboard data.
    ///
    /// The table name passed at construction time will be used. The function
    /// waits until the table is created before returning.
    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    pub async fn create_table(&self) -> crate::Result<()> {
        let output = self
            .client
            .create_table()
            .table_name(self.table_name.clone())
            .set_attribute_definitions(Some(vec![
                Self::attribute_definition(HASH_KEY, ScalarAttributeType::N),
                Self::attribute_definition(RANGE_KEY, ScalarAttributeType::N),
            ]))
            .set_key_schema(Some(vec![
                Self::key_schema_element(HASH_KEY, KeyType::Hash),
                Self::key_schema_element(RANGE_KEY, KeyType::Range),
            ]))
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await
            .map_err(|source| DynamoDbError::CreateTable {
                table_name: self.table_name.clone(),
                source: Box::new(source).into(),
            })?;

        self.wait_for_table_creation(&output).await
    }

    fn attribute_definition(
        attribute_name: &str,
        attribute_type: ScalarAttributeType,
    ) -> AttributeDefinition {
        AttributeDefinition::builder()
            .attribute_name(attribute_name)
            .attribute_type(attribute_type)
            .build()
            .expect("all attributes for attribution definition should be set")
    }

    fn key_schema_element(attribute_name: &str, key_type: KeyType) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(attribute_name)
            .key_type(key_type)
            .build()
            .expect("all attributes for key schema element should be set")
    }

    // Note: we disable code coverage for this method because there's no guarantee
    // the creation will take so long we'll have to wait, which means coverage might
    // be inconsistent between runs.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[tracing::instrument(skip_all, level = "trace", ret, err)]
    async fn wait_for_table_creation(
        &self,
        create_output: &CreateTableOutput,
    ) -> crate::Result<()> {
        let mut status = create_output
            .table_description()
            .and_then(TableDescription::table_status)
            .cloned();

        while let Some(TableStatus::Creating) = status {
            sleep(Duration::from_millis(100)).await;

            let output = self
                .client
                .describe_table()
                .table_name(self.table_name.clone())
                .send()
                .await
                .map_err(|source| DynamoDbError::CreateTable {
                    table_name: self.table_name.clone(),
                    source: Box::new(source).into(),
                })?;
            status = output
                .table()
                .and_then(TableDescription::table_status)
                .cloned();
        }

        Ok(())
    }
}

impl Storage for DynamoDbStorage {
    type Err = crate::Error;

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn load_previous(
        &self,
        year: i32,
        leaderboard_id: u64,
    ) -> Result<(Option<Leaderboard>, Option<ErrorKind>), Self::Err> {
        let load_previous_error =
            |source| DynamoDbError::LoadPreviousLeaderboard { leaderboard_id, year, source };

        Ok(self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key(HASH_KEY, AttributeValue::N(leaderboard_id.to_string()))
            .key(RANGE_KEY, AttributeValue::N(year.to_string()))
            .send()
            .await
            .map_err(|err| {
                load_previous_error(Box::new(err).into())
            })?
            .item
            .map(|item| {
                let data: Result<DynamoDbLeaderboardData, _> = serde_dynamo::from_item(item);
                data.map(|data| {
                    (data.leaderboard_data, data.last_error.map(|le| le.0))
                })
            })
            .transpose()
            .map(Option::unwrap_or_default)
            .map_err(|err| load_previous_error(err.into()))?)
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn save_success(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> Result<(), Self::Err> {
        let save_error = |source| DynamoDbError::SaveLeaderboard { leaderboard_id, year, source };

        let leaderboard_data = DynamoDbLeaderboardData::for_success(
            year,
            leaderboard_id,
            leaderboard.clone(),
        );
        let item = serde_dynamo::to_item(leaderboard_data).map_err(|err| save_error(err.into()))?;

        self.client
            .put_item()
            .table_name(self.table_name.clone())
            .set_item(Some(item))
            .send()
            .await
            .map_err(|err| save_error(Box::new(err).into()))?;

        Ok(())
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self), ret, err))]
    async fn save_error(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        error_kind: ErrorKind,
    ) -> Result<(), Self::Err> {
        let save_error = |source| DynamoDbError::SaveLastError { leaderboard_id, year, source };

        let last_error = DynamoDbLastErrorInformation(error_kind);
        let attribute_value = serde_dynamo::to_attribute_value(last_error)
            .map_err(|err| save_error(err.into()))?;

        self.client
            .update_item()
            .table_name(self.table_name.clone())
            .key(HASH_KEY, AttributeValue::N(leaderboard_id.to_string()))
            .key(RANGE_KEY, AttributeValue::N(year.to_string()))
            .update_expression("SET #last_error = :last_error")
            .expression_attribute_names("#last_error", LAST_ERROR)
            .expression_attribute_values(":last_error", attribute_value)
            .send()
            .await
            .map_err(|err| save_error(Box::new(err).into()))?;

        Ok(())
    }
}
