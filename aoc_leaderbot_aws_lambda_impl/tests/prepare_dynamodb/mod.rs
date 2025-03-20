use std::env;

use aoc_leaderbot_aws_lambda_impl::leaderbot::DEFAULT_DYNAMODB_TABLE_NAME;
use assert_cmd::Command;
use assert_matches::assert_matches;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableOutput;
use aws_sdk_dynamodb::types::{TableDescription, TableStatus};
use rstest::{fixture, rstest};
use serial_test::file_serial;
use testcontainers_modules::dynamodb_local::DynamoDb;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

const DYNAMODB_LOCAL_TAG: &str = "2.6.0";
const PREPARE_DYNAMODB_BIN_NAME: &str = "prepare_dynamodb";

#[fixture]
async fn local_dynamodb() -> (ContainerAsync<DynamoDb>, String) {
    let container = DynamoDb::default()
        .with_tag(DYNAMODB_LOCAL_TAG)
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(8000).await.unwrap();
    (container, format!("http://{host}:{port}"))
}

async fn check_table_exists(table_name: &str, endpoint_url: &str) {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region("ca-central-1")
        .test_credentials()
        .endpoint_url(endpoint_url)
        .load()
        .await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let result = client.describe_table().table_name(table_name).send().await;
    assert_matches!(result, Ok(DescribeTableOutput { table, .. }) => {
        assert_matches!(table, Some(TableDescription { table_status, .. }) => {
            assert_eq!(table_status, Some(TableStatus::Active));
        });
    });
}

#[derive(Debug)]
struct RemoveAwsEnvVars {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
}

impl Default for RemoveAwsEnvVars {
    fn default() -> Self {
        let access_key_id = env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").ok();

        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");

        Self { access_key_id, secret_access_key }
    }
}

impl Drop for RemoveAwsEnvVars {
    fn drop(&mut self) {
        if let Some(access_key_id) = self.access_key_id.take() {
            env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
        }
        if let Some(secret_access_key) = self.secret_access_key.take() {
            env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
        }
    }
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
#[file_serial(testcontainers_dynamodb)]
async fn with_default_table_name(#[future] local_dynamodb: (ContainerAsync<DynamoDb>, String)) {
    let (_dynamodb, endpoint_url) = local_dynamodb;

    Command::cargo_bin(PREPARE_DYNAMODB_BIN_NAME)
        .unwrap()
        .arg("--test-endpoint-url")
        .arg(&endpoint_url)
        .assert()
        .success();

    check_table_exists(DEFAULT_DYNAMODB_TABLE_NAME, &endpoint_url).await;
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
#[file_serial(testcontainers_dynamodb)]
async fn with_custom_table_name(#[future] local_dynamodb: (ContainerAsync<DynamoDb>, String)) {
    let (_dynamodb, endpoint_url) = local_dynamodb;

    let table_name = "aoc_leaderbot_test_table_name";

    Command::cargo_bin(PREPARE_DYNAMODB_BIN_NAME)
        .unwrap()
        .arg("--table-name")
        .arg(table_name)
        .arg("--test-endpoint-url")
        .arg(&endpoint_url)
        .assert()
        .success();

    check_table_exists(table_name, &endpoint_url).await;
}

#[test_log::test]
#[file_serial(aws_env)]
fn without_connection() {
    let _remove_aws_env_vars = RemoveAwsEnvVars::default();

    Command::cargo_bin(PREPARE_DYNAMODB_BIN_NAME)
        .unwrap()
        .assert()
        .failure();
}
