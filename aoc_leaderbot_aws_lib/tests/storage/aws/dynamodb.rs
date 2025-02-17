// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod dynamo_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_aws_lib::error::{
        CreateDynamoDbTableError, DynamoDbError, LoadPreviousDynamoDbError, SaveDynamoDbError,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::{
        DynamoDbLeaderboardData, DynamoDbStorage, HASH_KEY, LEADERBOARD_DATA, RANGE_KEY,
    };
    use aoc_leaderbot_lib::leaderbot::Storage;
    use aoc_leaderbot_test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};
    use assert_matches::assert_matches;
    use aws_config::BehaviorVersion;
    use aws_sdk_dynamodb::types::AttributeValue;
    use aws_sdk_dynamodb::Client;
    use uuid::Uuid;

    const LOCAL_ENDPOINT_URL: &str = "http://localhost:8000";

    struct LocalTable {
        name: String,
        client: Client,
        storage: DynamoDbStorage,
    }

    impl LocalTable {
        pub async fn without_table() -> Self {
            let name = Self::random_table_name();

            let config = aws_config::defaults(BehaviorVersion::latest())
                .region("ca-central-1")
                .test_credentials()
                .endpoint_url(LOCAL_ENDPOINT_URL)
                .load()
                .await;

            let client = Client::new(&config);
            let storage = DynamoDbStorage::with_config(&config, name.clone()).await;

            Self { name, client, storage }
        }

        pub async fn with_table() -> Self {
            let mut table = Self::without_table().await;
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

        pub fn storage(&mut self) -> &mut DynamoDbStorage {
            &mut self.storage
        }

        pub async fn save_leaderboard(&self, leaderboard: &Leaderboard) {
            let leaderboard_data = DynamoDbLeaderboardData {
                leaderboard_id: LEADERBOARD_ID,
                year: YEAR,
                leaderboard_data: leaderboard.clone(),
            };
            let item = serde_dynamo::to_item(leaderboard_data).unwrap();

            self.client()
                .put_item()
                .table_name(self.name())
                .set_item(Some(item))
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
                .item
                .map(|item| {
                    let data: DynamoDbLeaderboardData = serde_dynamo::from_item(item).unwrap();
                    data.leaderboard_data
                })
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

            #[tokio::test]
            async fn without_data() {
                let mut table = LocalTable::with_table().await;
                let previous_leaderboard =
                    table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                assert_matches!(previous_leaderboard, Ok(None));
            }

            #[tokio::test]
            async fn with_data() {
                let mut table = LocalTable::with_table().await;
                let expected_leaderboard = get_sample_leaderboard();
                table.save_leaderboard(&expected_leaderboard).await;

                let previous_leaderboard =
                    table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                assert_matches!(previous_leaderboard, Ok(Some(actual_leaderboard)) if actual_leaderboard == expected_leaderboard);
            }

            pub mod errors {
                use super::*;

                #[tokio::test]
                async fn get_item() {
                    let mut table = LocalTable::without_table().await;
                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoDbError::GetItem(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                #[tokio::test]
                async fn missing_leaderboard_data() {
                    let mut table = LocalTable::with_table().await;
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
                            DynamoDbError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoDbError::Deserialize(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                #[tokio::test]
                async fn invalid_leaderboard_data_type() {
                    let mut table = LocalTable::with_table().await;
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
                            DynamoDbError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoDbError::Deserialize(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                #[tokio::test]
                async fn parse_error() {
                    let mut table = LocalTable::with_table().await;
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
                            DynamoDbError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoDbError::Deserialize(_),
                            }
                        )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }
            }
        }

        pub mod save {
            use super::*;

            #[tokio::test]
            async fn without_existing() {
                let expected_leaderboard = get_sample_leaderboard();

                let mut table = LocalTable::with_table().await;
                table
                    .storage()
                    .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                    .await
                    .unwrap();

                let actual_leaderboard = table.load_leaderboard().await;
                assert_eq!(expected_leaderboard, actual_leaderboard);
            }

            #[tokio::test]
            async fn with_existing() {
                let mut table = LocalTable::with_table().await;
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

                #[tokio::test]
                async fn put_item() {
                    let mut table = LocalTable::without_table().await;
                    let leaderboard = get_sample_leaderboard();
                    let save_result = table
                        .storage()
                        .save(YEAR, LEADERBOARD_ID, &leaderboard)
                        .await;
                    assert_matches!(
                        save_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::SaveLeaderboard {
                                leaderboard_id,
                                year,
                                source: SaveDynamoDbError::PutItem(_),
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

                #[tokio::test]
                async fn create_table() {
                    let mut table = LocalTable::with_table().await;
                    let create_result = table.storage().create_table().await;
                    assert_matches!(
                        create_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::CreateTable {
                                table_name: actual_table_name,
                                source: CreateDynamoDbTableError::CreateTable(_),
                            }
                        )) if actual_table_name == table.name()
                    );
                }
            }
        }
    }
}
