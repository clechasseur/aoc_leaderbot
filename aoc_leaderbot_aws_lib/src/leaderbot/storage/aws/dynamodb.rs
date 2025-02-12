//! [`leaderbot::Storage`](Storage) keeping data in an [AWS DynamoDB] table.
//!
//! [AWS DynamoDB]: https://aws.amazon.com/dynamodb/

use std::time::Duration;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderbot_lib::leaderbot::Storage;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::operation::create_table::CreateTableOutput;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType,
    ScalarAttributeType, TableDescription, TableStatus,
};
use aws_sdk_dynamodb::Client;
use tokio::time::sleep;

use crate::error::{DynamoDbError, LoadPreviousDynamoDbError};

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

/// Bot storage that keeps data in an [AWS DynamoDB] table.
///
/// [AWS DynamoDB]: https://aws.amazon.com/dynamodb/
#[derive(Debug, Clone)]
pub struct DynamoDbStorage {
    client: Client,
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
        Self { client: Client::new(config), table_name: table_name.into() }
    }

    /// Creates a DynamoDB table suitable for storing leaderboard data.
    ///
    /// The table name passed at construction time will be used. The function
    /// waits until the table is created before returning.
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
                source: source.into(),
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
                    source: source.into(),
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

    async fn load_previous(
        &self,
        year: i32,
        leaderboard_id: u64,
    ) -> Result<Option<Leaderboard>, Self::Err> {
        let load_previous_error =
            |source| DynamoDbError::LoadPreviousLeaderboard { leaderboard_id, year, source };

        let output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key(HASH_KEY, AttributeValue::N(leaderboard_id.to_string()))
            .key(RANGE_KEY, AttributeValue::N(year.to_string()))
            .send()
            .await
            .map_err(|err| load_previous_error(err.into()))?;

        match output.item() {
            None => Ok(None),
            Some(item) => match item.get(LEADERBOARD_DATA) {
                None => {
                    Err(load_previous_error(LoadPreviousDynamoDbError::MissingLeaderboardData)
                        .into())
                },
                Some(AttributeValue::S(data)) => {
                    let leaderboard = serde_json::from_str(data)
                        .map_err(|err| load_previous_error(err.into()))?;
                    Ok(Some(leaderboard))
                },
                Some(_) => {
                    Err(load_previous_error(LoadPreviousDynamoDbError::InvalidLeaderboardDataType)
                        .into())
                },
            },
        }
    }

    async fn save(
        &mut self,
        year: i32,
        leaderboard_id: u64,
        leaderboard: &Leaderboard,
    ) -> Result<(), Self::Err> {
        let save_error = |source| DynamoDbError::SaveLeaderboard { leaderboard_id, year, source };

        let leaderboard_data =
            serde_json::to_string(&leaderboard).map_err(|err| save_error(err.into()))?;
        self.client
            .put_item()
            .table_name(self.table_name.clone())
            .item(HASH_KEY, AttributeValue::N(leaderboard_id.to_string()))
            .item(RANGE_KEY, AttributeValue::N(year.to_string()))
            .item(LEADERBOARD_DATA, AttributeValue::S(leaderboard_data))
            .send()
            .await
            .map_err(|err| save_error(err.into()))?;

        Ok(())
    }
}
