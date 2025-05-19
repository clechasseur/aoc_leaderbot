//! Test helpers for [`DynamoDbStorage`].
//!
//! Not meant to be used outside the project; no guarantee on API stability.

use std::future::Future;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderboard::test_helpers::{TEST_LEADERBOARD_ID, TEST_YEAR};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use rstest::fixture;
use uuid::Uuid;

use crate::leaderbot::storage::aws::dynamodb::{
    DynamoDbLeaderboardData, DynamoDbStorage, HASH_KEY, RANGE_KEY,
};

/// Endpoint URL for a locally-running DynamoDB.
pub const LOCAL_ENDPOINT_URL: &str = "http://localhost:8000";

/// Wrapper for a test DynamoDB table stored in a local DynamoDB,
/// suitable for testing [`DynamoDbStorage`].
///
/// # Notes
///
/// Because this is meant to be used for testing, most methods to
/// not return `Result`s and simply panic if something fails.
#[derive(Debug, Clone)]
pub struct LocalTable {
    name: String,
    client: aws_sdk_dynamodb::Client,
    storage: DynamoDbStorage,
}

impl LocalTable {
    /// Creates a [`LocalTable`] wrapping a [`DynamoDbStorage`].
    ///
    /// Does not create the test table itself; to create it later,
    /// call [`create`]. If the table is required right away,
    /// you can call [`with_table`] instead.
    ///
    /// [`create`]: Self::create
    /// [`with_table`]: Self::with_table
    pub async fn without_table() -> Self {
        let name = Self::random_table_name();

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region("ca-central-1")
            .test_credentials()
            .endpoint_url(LOCAL_ENDPOINT_URL)
            .load()
            .await;

        let client = aws_sdk_dynamodb::Client::new(&config);
        let storage = DynamoDbStorage::with_config(&config, name.clone()).await;

        Self { name, client, storage }
    }

    /// Creates a [`LocalTable`] wrapping a [`DynamoDbStorage`],
    /// creating the test table right away.
    pub async fn with_table() -> Self {
        let table = Self::without_table().await;
        table.create().await;
        table
    }

    /// Creates the test DynamoDB table.
    ///
    /// Call this only if the table hasn't been created yet,
    /// i.e. if [`without_table`] was called, and only once.
    ///
    /// [`without_table`]: Self::without_table
    pub async fn create(&self) {
        self.storage
            .create_table()
            .await
            .expect("test table should be creatable");
    }

    /// Drops the test table.
    ///
    /// Call this after testing is done to ensure the test table
    /// is removed from DynamoDB. Do not call this unless the
    /// table has been created, either because [`with_table`]
    /// has been used or because [`create`] has been called.
    ///
    /// # Notes
    ///
    /// This is not done by implementing `Drop` because it needs
    /// to be asynchronous. For an easier way to use this method
    /// in a testing context, see [`run_test`].
    ///
    /// [`with_table`]: Self::with_table
    /// [`create`]: Self::create
    /// [`run_test`]: Self::run_test
    pub async fn drop(&self) {
        self.client
            .delete_table()
            .table_name(self.name())
            .send()
            .await
            .expect("test table should be deletable");
    }

    /// Returns the name of the test table.
    ///
    /// Test table names are generated randomly.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the [DynamoDB client]
    /// used by this wrapper for direct DynamoDB operations.
    ///
    /// [DynamoDB client]: aws_sdk_dynamodb::Client
    pub fn client(&self) -> &aws_sdk_dynamodb::Client {
        &self.client
    }

    /// Returns a reference to the wrapped [`DynamoDbStorage`].
    pub fn storage(&mut self) -> &mut DynamoDbStorage {
        &mut self.storage
    }

    /// Saves the given [`Leaderboard`] to the test table.
    ///
    /// The leaderboard will be associated with the test
    /// values [`LEADERBOARD_ID`] and [`YEAR`].
    pub async fn save_leaderboard(&self, leaderboard: &Leaderboard) {
        let leaderboard_data = DynamoDbLeaderboardData::for_success(
            TEST_YEAR,
            TEST_LEADERBOARD_ID,
            leaderboard.clone(),
        );
        let item = serde_dynamo::to_item(leaderboard_data)
            .expect("leaderboard data should be serializable");

        self.client()
            .put_item()
            .table_name(self.name())
            .set_item(Some(item))
            .send()
            .await
            .expect("leaderboard data should be storable in the test table");
    }

    /// Loads a [`Leaderboard`] from the test table directly,
    /// using the test values [`TEST_LEADERBOARD_ID`] and [`TEST_YEAR`].
    ///
    /// Loads the data from the table through the DynamoDB client,
    /// not via the [`DynamoDbStorage`] wrapper.
    ///
    /// Panics if there was no leaderboard data in the table.
    pub async fn load_leaderboard(&self) -> Leaderboard {
        self.client()
            .get_item()
            .table_name(self.name())
            .key(HASH_KEY, AttributeValue::N(TEST_LEADERBOARD_ID.to_string()))
            .key(RANGE_KEY, AttributeValue::N(TEST_YEAR.to_string()))
            .send()
            .await
            .expect("leaderboard data should exist in the test table")
            .item
            .and_then(|item| {
                let data: DynamoDbLeaderboardData = serde_dynamo::from_item(item)
                    .expect("leaderboard data should be deserializable");
                data.leaderboard_data
            })
            .expect("leaderboard data should exist in the test table")
    }

    /// Creates a test table wrapper, calls the provided
    /// test function with it and ensures it is dropped
    /// before returning.
    ///
    /// # Notes
    ///
    /// This function is not `async`, so it must be called
    /// from within a regular test, not a `tokio` test.
    /// The function passed to this method, however, must
    /// return a `Future`. The easiest way is to use an
    /// `async` block; example:
    ///
    /// ```
    /// # use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::LocalTable;
    /// #[test]
    /// # #[cfg(feature = "__testing")]
    /// fn some_test() {
    ///     LocalTable::run_test(|table| async move {
    ///         // Run some tests with table here...
    ///         assert!(!table.name().is_empty());
    ///     });
    /// }
    /// ```
    pub fn run_test<TF, TFR>(test_f: TF)
    where
        TF: FnOnce(Self) -> TFR,
        TFR: Future<Output = ()> + Send + 'static,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("should be able to create a Tokio runtime for testing");

        let table = runtime.block_on(Self::with_table());

        let test_table = table.clone();
        let result = runtime.block_on(runtime.spawn(test_f(test_table)));

        runtime.block_on(table.drop());
        result.unwrap();
    }

    fn random_table_name() -> String {
        format!("aoc_leaderbot_aws_test_table_{}", Uuid::new_v4())
    }
}

#[fixture]
pub async fn local_non_existent_table() -> LocalTable {
    LocalTable::without_table().await
}

#[fixture]
pub async fn local_table() -> LocalTable {
    LocalTable::with_table().await
}
