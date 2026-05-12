//! Test helpers for [`DynamoDbStorage`].
//!
//! Not meant to be used outside the `aoc_leaderbot` projects; no guarantee on API stability.

use std::future::Future;
use std::sync::Arc;

use aoc_leaderboard::aoc::Leaderboard;
use aoc_leaderboard::test_helpers::{TEST_LEADERBOARD_ID, TEST_YEAR};
use aoc_leaderbot_lib::ErrorKind;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use testcontainers_modules::dynamodb_local::DynamoDb;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tokio::spawn;
use uuid::Uuid;

use crate::leaderbot::storage::aws::dynamodb::{
    DynamoDbLeaderboardData, DynamoDbStorage, HASH_KEY, LAST_ERROR, RANGE_KEY,
};

pub const LOCAL_ENDPOINT_URL: &str = "http://localhost:8000";
pub const DYNAMODB_LOCAL_TAG: &str = "3.3.0";

#[derive(Debug, Clone, Builder)]
#[builder(name = "LocalTableBuilder", derive(Debug), build_fn(private, name = "build_internal"))]
pub struct LocalTableConfig {
    #[builder(default = "true")]
    pub pre_create: bool,
    #[builder(default)]
    pub containerized: bool,
    #[builder(default, setter(into, strip_option))]
    pub name: Option<String>,
}

impl Default for LocalTableConfig {
    fn default() -> Self {
        Self { pre_create: true, containerized: false, name: None }
    }
}

impl LocalTableBuilder {
    pub fn build_config(&self) -> LocalTableConfig {
        self.build_internal()
            .expect("all local table fields should have default values")
    }

    pub async fn build(&self) -> LocalTable {
        LocalTable::new(self.build_config()).await
    }

    pub fn run_test<TF, TFR>(&self, test_f: TF)
    where
        TF: FnOnce(LocalTable) -> TFR + Send + 'static,
        TFR: Future<Output = ()> + Send + 'static,
    {
        LocalTable::run_test(Some(self.build_config()), test_f);
    }
}

#[derive(Debug, Clone)]
pub struct LocalTable {
    name: String,
    endpoint_url: String,
    _container: Option<Arc<ContainerAsync<DynamoDb>>>,
    client: aws_sdk_dynamodb::Client,
    storage: DynamoDbStorage,
}

impl LocalTable {
    pub fn builder() -> LocalTableBuilder {
        <_>::default()
    }

    pub async fn new(config: LocalTableConfig) -> Self {
        let name = config.name.unwrap_or_else(Self::random_table_name);

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
            _container: container,
            client: aws_sdk_dynamodb::Client::new(&sdk_config),
            storage: DynamoDbStorage::with_config(&sdk_config, name).await,
        };

        if config.pre_create {
            table.create().await;
        }

        table
    }

    pub async fn default() -> Self {
        Self::new(<_>::default()).await
    }

    pub async fn create(&self) {
        self.storage
            .create_table(None)
            .await
            .expect("test table should be creatable");
    }

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
            Err(err) => {
                // This will be called during unwinding, so panicking here would probably be bad.
                eprintln!("failed to delete test table {}: {err}", self.name);
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dynamodb_endpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn client(&self) -> &aws_sdk_dynamodb::Client {
        &self.client
    }

    pub fn storage(&mut self) -> &mut DynamoDbStorage {
        &mut self.storage
    }

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

    pub fn run_test<TF, TFR>(config: Option<LocalTableConfig>, test_f: TF)
    where
        TF: FnOnce(Self) -> TFR + Send + 'static,
        TFR: Future<Output = ()> + Send + 'static,
    {
        let config = config.unwrap_or_default();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("should be able to create a Tokio runtime for testing")
            .block_on(async move {
                let table = Self::new(config).await;
                let result = spawn(test_f(table.clone())).await;

                table.drop().await;
                result.unwrap();
            });
    }

    fn random_table_name() -> String {
        format!("aoc_leaderbot_aws_test_table_{}", Uuid::new_v4())
    }

    async fn create_container() -> (Arc<ContainerAsync<DynamoDb>>, String) {
        let container = Arc::new(
            DynamoDb::default()
                .with_tag(DYNAMODB_LOCAL_TAG)
                .start()
                .await
                .expect("should be able to create dynamodb-local container"),
        );

        let host = container
            .get_host()
            .await
            .expect("should be able to get dynamodb-local container host");
        let port = container
            .get_host_port_ipv4(8000)
            .await
            .expect("should be able to get dynamodb-local container port");
        (container, format!("http://{host}:{port}"))
    }
}

// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(all(test, any(not(ci), target_os = "linux")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use aoc_leaderboard::test_helpers::test_leaderboard;
    use assert_matches::assert_matches;
    use rstest::{fixture, rstest};
    use serial_test::file_serial;

    use super::*;

    #[fixture]
    fn default_config() -> LocalTableConfig {
        LocalTableConfig::default()
    }

    #[fixture]
    fn not_pre_created_config() -> LocalTableConfig {
        LocalTable::builder().pre_create(false).build_config()
    }

    #[fixture]
    fn containerized_config() -> LocalTableConfig {
        LocalTable::builder().containerized(true).build_config()
    }

    #[fixture]
    fn custom_table_name_config() -> LocalTableConfig {
        LocalTable::builder()
            .name(LocalTable::random_table_name())
            .build_config()
    }

    mod local_table_struct {
        use super::*;

        mod lifecycle {
            use super::*;

            fn lifecycle_test(config: LocalTableConfig, leaderboard: Leaderboard) {
                let should_exist = config.pre_create;
                LocalTable::run_test(Some(config), move |table| async move {
                    assert_eq!(table.exists().await, should_exist);

                    if !should_exist {
                        table.create().await;
                        assert!(table.exists().await);
                    }

                    table.save_leaderboard(&leaderboard).await;
                    let (actual_leaderboard, error_kind) =
                        table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual_leaderboard, Some(actual_leaderboard) => {
                        assert_eq!(actual_leaderboard, leaderboard);
                    });
                    assert!(error_kind.is_none());

                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;
                    let (actual_leaderboard, error_kind) =
                        table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual_leaderboard, Some(actual_leaderboard) => {
                        assert_eq!(actual_leaderboard, leaderboard);
                    });
                    assert_matches!(
                        error_kind,
                        Some(ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess))
                    );
                });
            }

            #[rstest]
            #[test_log::test]
            fn default(
                #[from(default_config)] config: LocalTableConfig,
                #[from(test_leaderboard)] leaderboard: Leaderboard,
            ) {
                lifecycle_test(config, leaderboard);
            }

            #[rstest]
            #[test_log::test]
            fn not_pre_created(
                #[from(not_pre_created_config)] config: LocalTableConfig,
                #[from(test_leaderboard)] leaderboard: Leaderboard,
            ) {
                lifecycle_test(config, leaderboard);
            }

            #[rstest]
            #[test_log::test]
            #[file_serial(testcontainers_dynamodb)]
            fn containerized(
                #[from(containerized_config)] config: LocalTableConfig,
                #[from(test_leaderboard)] leaderboard: Leaderboard,
            ) {
                lifecycle_test(config, leaderboard);
            }

            #[rstest]
            #[test_log::test]
            fn custom_table_name(
                #[from(custom_table_name_config)] config: LocalTableConfig,
                #[from(test_leaderboard)] leaderboard: Leaderboard,
            ) {
                lifecycle_test(config, leaderboard);
            }
        }
    }
}
