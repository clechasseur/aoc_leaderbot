// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod dynamo_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_aws_lib::error::{
        CreateDynamoTableError, DynamoError, LoadPreviousDynamoError, SaveDynamoError,
    };
    use aoc_leaderbot_aws_lib::leaderbot::aws::dynamo::storage::{
        DynamoStorage, HASH_KEY, LEADERBOARD_DATA, RANGE_KEY,
    };
    use aoc_leaderbot_lib::leaderbot::Storage;
    use aoc_leaderbot_test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};
    use assert_matches::assert_matches;
    use aws_config::SdkConfig;
    use aws_sdk_dynamodb::types::AttributeValue;
    use aws_sdk_dynamodb::Client;
    use testcontainers_modules::dynamodb_local::DynamoDb;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
    use uuid::Uuid;

    const DYNAMODB_LOCAL_TAG: &str = "2.5.4";

    #[derive(Debug)]
    struct LocalDynamo {
        _container: ContainerAsync<DynamoDb>,
        config: SdkConfig,
    }

    impl LocalDynamo {
        pub async fn new() -> Self {
            let container = DynamoDb::default()
                .with_tag(DYNAMODB_LOCAL_TAG)
                .start()
                .await
                .unwrap();
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(8000).await.unwrap();

            let config = aws_config::load_from_env()
                .await
                .to_builder()
                .endpoint_url(format!("http://{host}:{port}"))
                .build();

            Self { _container: container, config }
        }

        pub fn config(&self) -> &SdkConfig {
            &self.config
        }
    }

    struct LocalTable {
        name: String,
        client: Client,
        storage: DynamoStorage,
    }

    impl LocalTable {
        pub async fn without_table(local_dynamo: &LocalDynamo) -> Self {
            let name = Self::random_table_name();

            let client = Client::new(local_dynamo.config());
            let storage = DynamoStorage::with_config(local_dynamo.config(), name.clone()).await;

            Self { client, name, storage }
        }

        pub async fn with_table(local_dynamo: &LocalDynamo) -> Self {
            let mut table = Self::without_table(local_dynamo).await;
            table.create().await;
            table
        }

        pub async fn create(&mut self) {
            self.storage.create_table().await.unwrap();
        }

        pub fn name(&self) -> String {
            self.name.clone()
        }

        pub fn client(&self) -> &Client {
            &self.client
        }

        pub fn storage(&mut self) -> &mut DynamoStorage {
            &mut self.storage
        }

        pub async fn save_leaderboard(&self, leaderboard: &Leaderboard) {
            let leaderboard_data = serde_json::to_string(leaderboard).unwrap();

            self.client()
                .put_item()
                .table_name(self.name())
                .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                .item(LEADERBOARD_DATA, AttributeValue::S(leaderboard_data))
                .send()
                .await
                .unwrap();
        }

        pub async fn load_leaderboard(&self) -> Leaderboard {
            self.client()
                .get_item()
                .table_name(self.name())
                .key(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                .key(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                .send()
                .await
                .unwrap()
                .item()
                .unwrap()
                .get(LEADERBOARD_DATA)
                .unwrap()
                .as_s()
                .map(|s| serde_json::from_str(s).unwrap())
                .unwrap()
        }

        fn random_table_name() -> String {
            format!("aoc_leaderbot_aws_test_table_{}", Uuid::new_v4())
        }
    }

    mod storage_impl {
        use super::*;

        pub mod load_previous {
            use super::*;

            pub async fn without_data(local_dynamo: &LocalDynamo) {
                let mut table = LocalTable::with_table(local_dynamo).await;
                let previous_leaderboard =
                    table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                assert_matches!(previous_leaderboard, Ok(None));
            }

            pub async fn with_data(local_dynamo: &LocalDynamo) {
                let mut table = LocalTable::with_table(local_dynamo).await;
                let expected_leaderboard = get_sample_leaderboard();
                table.save_leaderboard(&expected_leaderboard).await;

                let previous_leaderboard =
                    table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                assert_matches!(previous_leaderboard, Ok(Some(actual_leaderboard)) if actual_leaderboard == expected_leaderboard);
            }

            pub mod errors {
                use super::*;

                pub async fn get_item(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::without_table(local_dynamo).await;
                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoError::GetItem(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                pub async fn missing_leaderboard_data(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::with_table(local_dynamo).await;
                    table
                        .client()
                        .put_item()
                        .table_name(table.name())
                        .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                        .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                        .send()
                        .await
                        .unwrap();

                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoError::MissingLeaderboardData,
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                pub async fn invalid_leaderboard_data_type(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::with_table(local_dynamo).await;
                    table
                        .client()
                        .put_item()
                        .table_name(table.name())
                        .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                        .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                        .item(LEADERBOARD_DATA, AttributeValue::N(42.to_string()))
                        .send()
                        .await
                        .unwrap();

                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoError::InvalidLeaderboardDataType,
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                pub async fn parse_error(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::with_table(local_dynamo).await;
                    table
                        .client()
                        .put_item()
                        .table_name(table.name())
                        .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                        .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                        .item(
                            LEADERBOARD_DATA,
                            AttributeValue::S("{\"hello\":\"world\"".to_string()),
                        )
                        .send()
                        .await
                        .unwrap();

                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoError::ParseError(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }
            }
        }

        pub mod save {
            use super::*;

            pub async fn without_existing(local_dynamo: &LocalDynamo) {
                let expected_leaderboard = get_sample_leaderboard();

                let mut table = LocalTable::with_table(local_dynamo).await;
                table
                    .storage()
                    .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                    .await
                    .unwrap();

                let actual_leaderboard = table.load_leaderboard().await;
                assert_eq!(expected_leaderboard, actual_leaderboard);
            }

            pub async fn with_existing(local_dynamo: &LocalDynamo) {
                let mut table = LocalTable::with_table(local_dynamo).await;
                let previous_leaderboard = get_sample_leaderboard();
                table.save_leaderboard(&previous_leaderboard).await;

                let expected_leaderboard = Leaderboard {
                    day1_ts: previous_leaderboard.day1_ts + 1,
                    ..previous_leaderboard
                };
                table
                    .storage()
                    .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                    .await
                    .unwrap();

                let actual_leaderboard = table.load_leaderboard().await;
                assert_eq!(expected_leaderboard, actual_leaderboard);
            }

            pub mod errors {
                use super::*;

                pub async fn put_item(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::without_table(local_dynamo).await;
                    let leaderboard = get_sample_leaderboard();
                    let save_result = table
                        .storage()
                        .save(YEAR, LEADERBOARD_ID, &leaderboard)
                        .await;
                    assert_matches!(
                        save_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::SaveLeaderboard {
                                leaderboard_id,
                                year,
                                source: SaveDynamoError::PutItem(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }
            }
        }

        pub mod create_table {
            use super::*;

            pub mod errors {
                use super::*;

                pub async fn create_table(local_dynamo: &LocalDynamo) {
                    let mut table = LocalTable::with_table(local_dynamo).await;
                    let create_result = table.storage().create_table().await;
                    assert_matches!(
                        create_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoError::CreateTable {
                                table_name: actual_table_name,
                                source: CreateDynamoTableError::CreateTable(_),
                            }
                        )) if actual_table_name == table.name()
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn all_tests() {
        let local_dynamo = LocalDynamo::new().await;

        storage_impl::load_previous::without_data(&local_dynamo).await;
        storage_impl::load_previous::with_data(&local_dynamo).await;

        storage_impl::load_previous::errors::get_item(&local_dynamo).await;
        storage_impl::load_previous::errors::missing_leaderboard_data(&local_dynamo).await;
        storage_impl::load_previous::errors::invalid_leaderboard_data_type(&local_dynamo).await;
        storage_impl::load_previous::errors::parse_error(&local_dynamo).await;

        storage_impl::save::without_existing(&local_dynamo).await;
        storage_impl::save::with_existing(&local_dynamo).await;

        storage_impl::save::errors::put_item(&local_dynamo).await;

        storage_impl::create_table::errors::create_table(&local_dynamo).await;
    }
}
