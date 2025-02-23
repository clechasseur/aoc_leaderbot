use aoc_leaderbot_aws_lambda_impl::leaderbot::DEFAULT_DYNAMODB_TABLE_NAME;
use assert_cmd::Command;
use assert_matches::assert_matches;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableOutput;
use aws_sdk_dynamodb::types::{TableDescription, TableStatus};
use serial_test::file_serial;
use testcontainers_modules::dynamodb_local::DynamoDb;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

const DYNAMODB_LOCAL_TAG: &str = "2.5.4";
const PREPARE_DYNAMODB_BIN_NAME: &str = "prepare_dynamodb";

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

#[test_log::test(tokio::test)]
#[file_serial(testcontainers_dynamodb)]
async fn with_default_table_name() {
    let (_dynamodb, endpoint_url) = local_dynamodb().await;

    Command::cargo_bin(PREPARE_DYNAMODB_BIN_NAME)
        .unwrap()
        .arg("--test-endpoint-url")
        .arg(&endpoint_url)
        .assert()
        .success();

    check_table_exists(DEFAULT_DYNAMODB_TABLE_NAME, &endpoint_url).await;
}

#[test_log::test(tokio::test)]
#[file_serial(testcontainers_dynamodb)]
async fn with_custom_table_name() {
    let (_dynamodb, endpoint_url) = local_dynamodb().await;

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
