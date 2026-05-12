use std::env;

use aoc_leaderbot_aws_lambda_impl::leaderbot::DEFAULT_DYNAMODB_TABLE_NAME;
use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::LocalTable;
use assert_cmd::{Command, cargo_bin};
use serial_test::{file_serial, serial};

#[derive(Debug)]
struct RemoveAwsEnvVars {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
}

impl Default for RemoveAwsEnvVars {
    fn default() -> Self {
        let access_key_id = env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").ok();

        unsafe {
            env::remove_var("AWS_ACCESS_KEY_ID");
            env::remove_var("AWS_SECRET_ACCESS_KEY");
        }

        Self { access_key_id, secret_access_key }
    }
}

impl Drop for RemoveAwsEnvVars {
    fn drop(&mut self) {
        unsafe {
            if let Some(access_key_id) = self.access_key_id.take() {
                env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
            }
            if let Some(secret_access_key) = self.secret_access_key.take() {
                env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
            }
        }
    }
}

#[test_log::test]
#[file_serial(testcontainers_dynamodb)]
fn with_default_table_name() {
    LocalTable::builder()
        .pre_create(false)
        .containerized(true)
        .name(DEFAULT_DYNAMODB_TABLE_NAME)
        .run_test(|table| async move {
            Command::new(cargo_bin!("prepare_dynamodb"))
                .arg("--test-endpoint-url")
                .arg(table.dynamodb_endpoint_url())
                .assert()
                .success();

            assert!(table.exists().await);
        });
}

#[test_log::test]
#[file_serial(testcontainers_dynamodb)]
fn with_custom_table_name() {
    let table_name = "aoc_leaderbot_test_table_name";

    LocalTable::builder()
        .pre_create(false)
        .containerized(true)
        .name(table_name)
        .run_test(move |table| async move {
            Command::new(cargo_bin!("prepare_dynamodb"))
                .arg("--table-name")
                .arg(table_name)
                .arg("--test-endpoint-url")
                .arg(table.dynamodb_endpoint_url())
                .assert()
                .success();

            assert!(table.exists().await);
        });
}

#[test_log::test]
#[file_serial(testcontainers_dynamodb)]
fn with_unconstrained_pay_per_request_billing_mode() {
    LocalTable::builder()
        .pre_create(false)
        .containerized(true)
        .name(DEFAULT_DYNAMODB_TABLE_NAME)
        .run_test(|table| async move {
            Command::new(cargo_bin!("prepare_dynamodb"))
                .arg("--billing-mode")
                .arg("pay-per-request")
                .arg("--test-endpoint-url")
                .arg(table.dynamodb_endpoint_url())
                .assert()
                .success();

            assert!(table.exists().await);
        });
}

#[test_log::test]
#[serial(aws_env)]
fn without_connection() {
    let _remove_aws_env_vars = RemoveAwsEnvVars::default();

    Command::new(cargo_bin!("prepare_dynamodb"))
        .assert()
        .failure();
}
