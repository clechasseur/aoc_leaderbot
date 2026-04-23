//! Test helpers for [`DynamoDbStorage`].
//!
//! Not meant to be used outside the project; no guarantee on API stability.

use std::future::Future;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderboard::test_helpers::{TEST_LEADERBOARD_ID, TEST_YEAR};
use aoc_leaderbot_lib::ErrorKind;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use rstest::fixture;
use testcontainers_modules::dynamodb_local::DynamoDb;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use uuid::Uuid;

use crate::leaderbot::storage::aws::dynamodb::{
    DynamoDbLeaderboardData, DynamoDbStorage, HASH_KEY, LAST_ERROR, RANGE_KEY,
};

/// Endpoint URL for a locally-running DynamoDB.
pub const LOCAL_ENDPOINT_URL: &str = "http://localhost:8000";

/// Tag of the DynamoDB Docker image used by [`LocalTable`] when [`containerized`].
///
/// [`containerized`]: LocalTableBuilder::containerized
pub const DYNAMODB_LOCAL_TAG: &str = "3.1.0";

/// Configuration used to create a [`LocalTable`].
///
/// # Example
///
/// ```
/// # use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::{LocalTable, LocalTableConfig};
///
/// let config = LocalTableConfig {
///     pre_create: false,
///     containerized: true,
///     name: Some("some_table".into()),
/// };
///
/// let table = LocalTable::new(config).await;
/// assert!(!table.exists().await);
///
/// table.create().await;
/// assert!(table.exists().await);
/// ```
#[derive(Debug, Clone, Builder)]
#[builder(name = "LocalTableBuilder", derive(Debug), build_fn(private, name = "build_internal"))]
#[builder_struct_attr(
    doc = r"
        Builder that can be used to create a [`LocalTable`].

        # Example
    "
)]
pub struct LocalTableConfig {
    #[builder(default = "true")]
    pre_create: bool,

    #[builder(default)]
    containerized: bool,

    #[builder(default, setter(into, strip_option))]
    name: Option<String>,
}

impl Default for LocalTableConfig {
    fn default() -> Self {
        Self {
            pre_create: true,
            containerized: false,
            name: None,
        }
    }
}

/// Wrapper for a test DynamoDB table stored in a local DynamoDB, suitable for testing
/// [`DynamoDbStorage`].
///
/// # Notes
///
/// Because this is meant to be used for testing, most methods to not return `Result` and simply
/// panic if something fails.
#[derive(Debug)]
pub struct LocalTable {
    name: String,
    endpoint_url: String,
    container: Option<ContainerAsync<DynamoDb>>,
    client: aws_sdk_dynamodb::Client,
    storage: DynamoDbStorage,
}

impl LocalTable {
    /// Returns a [builder](LocalTableBuilder) to construct a new [`LocalTable`].
    pub fn builder() -> LocalTableBuilder {
        <_>::default()
    }

    pub async fn new(config: LocalTableConfig) -> LocalTable {
        let name = config.name.unwrap_or_else(|| Self::random_table_name());

        let (container, endpoint_url) = match config.containerized {
            false => (None, LOCAL_ENDPOINT_URL.into()),
            true => {
                let (container, endpoint_url) = Self::create_container().await;
                (Some(container), endpoint_url)
            },
        };

        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region("ca-central-1")
            .test_credentials()
            .endpoint_url(endpoint_url.as_str())
            .load()
            .await;

        let table = LocalTable {
            name: name.clone(),
            endpoint_url,
            container,
            client: aws_sdk_dynamodb::Client::new(&sdk_config),
            storage: DynamoDbStorage::with_config(&sdk_config, name.as_str()).await,
        };

        if config.pre_create {
            table.create().await;
        }

        table
    }

    /// Creates the test DynamoDB table.
    ///
    /// Call this only if the table hasn't been created yet, i.e. if the builder's [`pre_create`]
    /// method was called with `false`.
    ///
    /// [`pre_create`]: LocalTableBuilder::pre_create
    pub async fn create(&self) {
        self.storage
            .create_table(None)
            .await
            .expect("test table should be creatable");
    }

    /// Checks if the test table exists.
    pub async fn exists(&self) -> bool {
        let describe_res = self
            .client
            .describe_table()
            .table_name(self.name())
            .send()
            .await;

        match describe_res {
            Ok(DescribeTableOutput { table: Some(_), .. }) => true,
            Ok(_) => false,
            Err(SdkError::ServiceError(service_err))
                if service_err.err().is_resource_not_found_exception() =>
            {
                false
            },
            Err(err) => panic!("failed to check if test table exists: {err}"),
        }
    }

    /// Drops the test table.
    ///
    /// Call this after testing is done to ensure the test table is removed from DynamoDB.
    /// Do not call this unless the table has been created, either because [`pre_create`]
    /// was set to `true` in the builder or because [`create`] has been called.
    ///
    /// # Notes
    ///
    /// This is not done by implementing `Drop` because it needs to be asynchronous. For an easier
    /// way to use this method in a testing context, see [`run_test`].
    ///
    /// [`pre_create`]: LocalTableBuilder::pre_create
    /// [`create`]: Self::create
    /// [`run_test`]: Self::run_test
    pub async fn drop(&self) {
        let delete_res = self
            .client
            .delete_table()
            .table_name(self.name())
            .send()
            .await;

        match delete_res {
            Ok(_) => (),
            Err(SdkError::ServiceError(service_err))
                if service_err.err().is_resource_not_found_exception() => {},
            Err(err) => panic!("failed to delete test table: {err}"),
        }
    }

    /// Returns the name of the test table.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the URL of the DynamoDB endpoint that this local table is connected to.
    pub fn dynamodb_endpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    /// Returns a reference to the [DynamoDB client] used by this wrapper for direct DynamoDB
    /// operations.
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
    /// The leaderboard will be associated with the test values [`TEST_LEADERBOARD_ID`] and
    /// [`TEST_YEAR`].
    ///
    /// Any existing data (including last error) will be overwritten.
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

    /// Saves the given [last error](ErrorKind) to the test table.
    ///
    /// The last error will be associated with the test values [`TEST_LEADERBOARD_ID`] and
    /// [`TEST_YEAR`].
    ///
    /// Any existing leaderboard data will be kept.
    pub async fn save_last_error(&self, error_kind: ErrorKind) {
        let attribute_value = serde_dynamo::to_attribute_value(error_kind)
            .expect("last error should be serializable");

        self.client()
            .update_item()
            .table_name(self.name())
            .key(HASH_KEY, AttributeValue::N(TEST_LEADERBOARD_ID.to_string()))
            .key(RANGE_KEY, AttributeValue::N(TEST_YEAR.to_string()))
            .update_expression("SET #last_error = :last_error")
            .expression_attribute_names("#last_error", LAST_ERROR)
            .expression_attribute_values(":last_error", attribute_value)
            .send()
            .await
            .expect("last error should be storable in the test table");
    }

    /// Loads a [`Leaderboard`] and any associated [last error](ErrorKind) from the test table
    /// directly, using the test values [`TEST_LEADERBOARD_ID`] and [`TEST_YEAR`].
    ///
    /// Loads the data from the table through the DynamoDB client, not via the
    /// [`DynamoDbStorage`] wrapper.
    pub async fn load_leaderboard_and_last_error(
        &self,
    ) -> (Option<Leaderboard>, Option<ErrorKind>) {
        self.client()
            .get_item()
            .table_name(self.name())
            .key(HASH_KEY, AttributeValue::N(TEST_LEADERBOARD_ID.to_string()))
            .key(RANGE_KEY, AttributeValue::N(TEST_YEAR.to_string()))
            .send()
            .await
            .expect("leaderboard data should be accessible")
            .item
            .map(|item| {
                let data: DynamoDbLeaderboardData = serde_dynamo::from_item(item)
                    .expect("leaderboard data should be deserializable");
                (data.leaderboard_data, data.last_error)
            })
            .unwrap_or_default()
    }

    /// Creates a test table wrapper, calls the provided test function with it and ensures it is
    /// dropped before returning.
    ///
    /// # Notes
    ///
    /// This function is not `async`, so it must be called from within a regular test, not a
    /// `tokio` test. The function passed to this method, however, must return a `Future`.
    /// The easiest way is to use an `async` block; example:
    ///
    /// ```no_run
    /// # use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::LocalTable;
    ///
    /// # #[cfg(feature = "__testing")]
    /// #[test]
    /// fn some_test() {
    ///     LocalTable::run_test(|table| async move {
    ///         assert!(table.exists().await);
    ///         // Run some tests with table here...
    ///     });
    /// }
    /// ```
    pub fn run_test<TF, TFR>(test_f: TF)
    where
        TF: FnOnce(Self) -> TFR,
        TFR: Future<Output = ()> + Send + 'static,
    {
        Self::internal_run_test(test_f, true);
    }

    /// Creates a test table wrapper without creating the table itself (using [`without_table`]),
    /// calls the provided test function with it and ensures it is dropped before returning if
    /// the table had been created by the test.
    ///
    /// # Notes
    ///
    /// This function is not `async`, so it must be called from within a regular test, not a
    /// `tokio` test. The function passed to this method, however, must return a `Future`.
    /// The easiest way is to use an `async` block; example:
    ///
    /// ```no_run
    /// # use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::LocalTable;
    ///
    /// # #[cfg(feature = "__testing")]
    /// #[test]
    /// fn some_test() {
    ///     LocalTable::run_test_without_table(|table| async move {
    ///         assert!(!table.exists().await);
    ///         // Run some tests with table here, maybe creating it...
    ///     });
    /// }
    /// ```
    ///
    /// [`without_table`]: Self::without_table
    pub fn run_test_without_table<TF, TFR>(test_f: TF)
    where
        TF: FnOnce(Self) -> TFR,
        TFR: Future<Output = ()> + Send + 'static,
    {
        Self::internal_run_test(test_f, false);
    }

    fn internal_run_test<TF, TFR>(test_f: TF, config: LocalTableConfig)
    where
        TF: FnOnce(Self) -> TFR,
        TFR: Future<Output = ()> + Send + 'static,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("should be able to create a Tokio runtime for testing");

        let table = runtime.block_on(Self::new(config));

        let test_table = table.clone();
        let result = runtime.block_on(runtime.spawn(test_f(test_table)));

        runtime.block_on(table.drop());
        result.unwrap();
    }

    fn random_table_name() -> String {
        format!("aoc_leaderbot_aws_test_table_{}", Uuid::new_v4())
    }

    async fn create_container() -> (ContainerAsync<DynamoDb>, String) {
        let container = DynamoDb::default()
            .with_tag(DYNAMODB_LOCAL_TAG)
            .start()
            .await
            .unwrap();

        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(8000).await.unwrap();
        (container, format!("http://{host}:{port}"))
    }
}

/// [`rstest`] fixture providing a [`LocalTable`] wrapper, but without any backing table.
///
/// Equivalent to [`LocalTable::without_table`].
#[fixture]
pub async fn local_non_existent_table() -> LocalTable {
    LocalTable::without_table().await
}

/// [`rstest`] fixture providing a [`LocalTable`] with a backing table.
///
/// Equivalent to [`LocalTable::with_table`].
#[fixture]
pub async fn local_table() -> LocalTable {
    LocalTable::with_table().await
}

/// [`rstest`] fixture
#[fixture]
pub async fn local_dynamodb() -> (ContainerAsync<DynamoDb>, String) {
    let container = DynamoDb::default()
        .with_tag(DYNAMODB_LOCAL_TAG)
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(8000).await.unwrap();
    (container, format!("http://{host}:{port}"))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    mod local_table {
        use super::*;

        #[test_log::test]
        fn lifecycle() {
            LocalTable::run_test_without_table(|mut table| async move {
                assert!(!table.exists().await);

                table.create().await;
                assert!(table.exists().await);


            });
        }
    }
}
