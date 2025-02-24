// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod dynamo_storage {
    use std::future::Future;

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
    use uuid::Uuid;

    const LOCAL_ENDPOINT_URL: &str = "http://localhost:8000";

    #[derive(Debug)]
    struct LocalTable {
        name: String,
        client: aws_sdk_dynamodb::Client,
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

            let client = aws_sdk_dynamodb::Client::new(&config);
            let storage = DynamoDbStorage::with_config(&config, name.clone()).await;

            Self { name, client, storage }
        }

        pub async fn with_table() -> Self {
            let table = Self::without_table().await;
            table.create().await;
            table
        }

        pub fn name(&self) -> String {
            self.name.clone()
        }

        pub fn client(&self) -> &aws_sdk_dynamodb::Client {
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

        async fn create(&self) {
            self.storage.create_table().await.unwrap();
        }
    }

    async fn drop_table<S>(client: &aws_sdk_dynamodb::Client, table_name: S)
    where
        S: Into<String>,
    {
        client
            .delete_table()
            .table_name(table_name)
            .send()
            .await
            .unwrap();
    }

    fn run_local_table_test<TF, TFR>(test_f: TF)
    where
        TF: FnOnce(LocalTable) -> TFR,
        TFR: Future<Output = ()> + Send + 'static,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let table = runtime.block_on(LocalTable::with_table());
        let table_name = table.name();
        let client = table.client().clone();

        let result = runtime.block_on(runtime.spawn(test_f(table)));

        runtime.block_on(drop_table(&client, table_name));
        result.unwrap();
    }

    mod storage_impl {
        use super::*;

        pub mod load_previous {
            use super::*;

            #[test_log::test]
            fn without_data() {
                run_local_table_test(|mut table| async move {
                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(previous_leaderboard, Ok(None));
                });
            }

            #[test_log::test]
            fn with_data() {
                run_local_table_test(|mut table| async move {
                    let expected_leaderboard = get_sample_leaderboard();
                    table.save_leaderboard(&expected_leaderboard).await;

                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(previous_leaderboard, Ok(Some(actual_leaderboard)) if actual_leaderboard == expected_leaderboard);
                });
            }

            pub mod errors {
                use super::*;

                #[test_log::test(tokio::test)]
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

                #[test_log::test]
                fn missing_leaderboard_data() {
                    run_local_table_test(|mut table| async move {
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
                    });
                }

                #[test_log::test]
                fn invalid_leaderboard_data_type() {
                    run_local_table_test(|mut table| async move {
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
                    });
                }

                #[test_log::test]
                fn parse_error() {
                    run_local_table_test(|mut table| async move {
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
                    });
                }
            }
        }

        pub mod save {
            use super::*;

            #[test_log::test]
            fn without_existing() {
                run_local_table_test(|mut table| async move {
                    let expected_leaderboard = get_sample_leaderboard();
                    table
                        .storage()
                        .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                        .await
                        .unwrap();

                    let actual_leaderboard = table.load_leaderboard().await;
                    assert_eq!(expected_leaderboard, actual_leaderboard);
                });
            }

            #[test_log::test]
            fn with_existing() {
                run_local_table_test(|mut table| async move {
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
                });
            }

            pub mod errors {
                use super::*;

                #[test_log::test(tokio::test)]
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

                #[test_log::test]
                fn create_table() {
                    run_local_table_test(|mut table| async move {
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
                    });
                }
            }
        }
    }
}
