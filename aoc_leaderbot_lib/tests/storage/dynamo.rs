mod dynamo_storage {
    use std::future::Future;
    use std::panic::{resume_unwind, AssertUnwindSafe, UnwindSafe};

    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_lib::error::{
        AwsError, CreateDynamoTableError, DynamoError, LoadPreviousDynamoError, SaveDynamoError,
    };
    use aoc_leaderbot_lib::leaderbot::storage::dynamo::{
        DynamoStorage, HASH_KEY, LEADERBOARD_DATA, RANGE_KEY,
    };
    use aoc_leaderbot_lib::leaderbot::Storage;
    use assert_matches::assert_matches;
    use aws_sdk_dynamodb::types::AttributeValue;
    use aws_sdk_dynamodb::Client;
    use futures::FutureExt;
    use uuid::Uuid;

    use crate::test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};

    async fn dynamo_client() -> Client {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    }

    fn random_table_name() -> String {
        format!("aoc_leaderbot_lib_test_table_{}", Uuid::new_v4())
    }

    struct TestTable(String);

    impl TestTable {
        async fn new() -> Self {
            let table_name = random_table_name();

            let storage = DynamoStorage::new(table_name.clone()).await;
            storage.create_table().await.unwrap();

            Self(table_name)
        }

        fn table_name(&self) -> String {
            self.0.clone()
        }

        async fn drop(&self) {
            dynamo_client()
                .await
                .delete_table()
                .table_name(self.0.clone())
                .send()
                .await
                .unwrap();
        }
    }

    async fn run_test<F, R>(f: F)
    where
        F: FnOnce(String) -> R,
        R: Future<Output = ()> + UnwindSafe,
    {
        let test_table = TestTable::new().await;

        let test_result = f(test_table.table_name()).catch_unwind().await;

        test_table.drop().await;

        if let Err(err) = test_result {
            resume_unwind(err);
        }
    }

    async fn save_leaderboard(table_name: String, leaderboard: &Leaderboard) {
        let leaderboard_data = serde_json::to_string(leaderboard).unwrap();

        dynamo_client()
            .await
            .put_item()
            .table_name(table_name)
            .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
            .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
            .item(LEADERBOARD_DATA, AttributeValue::S(leaderboard_data))
            .send()
            .await
            .unwrap();
    }

    async fn load_leaderboard(table_name: String) -> Leaderboard {
        dynamo_client()
            .await
            .get_item()
            .table_name(table_name)
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

    mod storage_impl {
        use super::*;

        mod load_previous {
            use super::*;

            #[tokio::test]
            async fn without_data() {
                let test = |table_name| {
                    AssertUnwindSafe(async {
                        let storage = DynamoStorage::new(table_name).await;

                        let previous_leaderboard =
                            storage.load_previous(YEAR, LEADERBOARD_ID).await;
                        assert_matches!(previous_leaderboard, Ok(None));
                    })
                };

                run_test(test).await;
            }

            #[tokio::test]
            async fn with_data() {
                let test = |table_name: String| {
                    AssertUnwindSafe(async {
                        let expected_leaderboard = get_sample_leaderboard();
                        save_leaderboard(table_name.clone(), &expected_leaderboard).await;

                        let storage = DynamoStorage::new(table_name).await;

                        let previous_leaderboard =
                            storage.load_previous(YEAR, LEADERBOARD_ID).await;
                        assert_matches!(previous_leaderboard, Ok(Some(actual_leaderboard)) if actual_leaderboard == expected_leaderboard);
                    })
                };

                run_test(test).await;
            }

            mod errors {
                use super::*;

                #[tokio::test]
                async fn get_item() {
                    let table_name = random_table_name();
                    let storage = DynamoStorage::new(table_name).await;

                    let previous_leaderboard = storage.load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_lib::Error::Aws(
                            AwsError::Dynamo(
                                DynamoError::LoadPreviousLeaderboard {
                                    leaderboard_id,
                                    year,
                                    source: LoadPreviousDynamoError::GetItem(_),
                                }
                            ))
                        ) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }

                #[tokio::test]
                async fn missing_leaderboard_data() {
                    let test = |table_name: String| {
                        AssertUnwindSafe(async {
                            dynamo_client()
                                .await
                                .put_item()
                                .table_name(table_name.clone())
                                .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                                .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                                .send()
                                .await
                                .unwrap();

                            let storage = DynamoStorage::new(table_name).await;

                            let previous_leaderboard =
                                storage.load_previous(YEAR, LEADERBOARD_ID).await;
                            assert_matches!(
                                previous_leaderboard,
                                Err(aoc_leaderbot_lib::Error::Aws(
                                    AwsError::Dynamo(
                                        DynamoError::LoadPreviousLeaderboard {
                                            leaderboard_id,
                                            year,
                                            source: LoadPreviousDynamoError::MissingLeaderboardData,
                                        }
                                    )
                                )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                            );
                        })
                    };

                    run_test(test).await;
                }

                #[tokio::test]
                async fn invalid_leaderboard_data_type() {
                    let test = |table_name: String| {
                        AssertUnwindSafe(async {
                            dynamo_client()
                                .await
                                .put_item()
                                .table_name(table_name.clone())
                                .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                                .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                                .item(LEADERBOARD_DATA, AttributeValue::N(42.to_string()))
                                .send()
                                .await
                                .unwrap();

                            let storage = DynamoStorage::new(table_name).await;

                            let previous_leaderboard =
                                storage.load_previous(YEAR, LEADERBOARD_ID).await;
                            assert_matches!(
                                previous_leaderboard,
                                Err(aoc_leaderbot_lib::Error::Aws(
                                    AwsError::Dynamo(
                                        DynamoError::LoadPreviousLeaderboard {
                                            leaderboard_id,
                                            year,
                                            source: LoadPreviousDynamoError::InvalidLeaderboardDataType,
                                        }
                                    )
                                )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                            );
                        })
                    };

                    run_test(test).await;
                }

                #[tokio::test]
                async fn parse_error() {
                    let test = |table_name: String| {
                        AssertUnwindSafe(async {
                            dynamo_client()
                                .await
                                .put_item()
                                .table_name(table_name.clone())
                                .item(HASH_KEY, AttributeValue::N(LEADERBOARD_ID.to_string()))
                                .item(RANGE_KEY, AttributeValue::N(YEAR.to_string()))
                                .item(
                                    LEADERBOARD_DATA,
                                    AttributeValue::S("{\"hello\":\"world\"".to_string()),
                                )
                                .send()
                                .await
                                .unwrap();

                            let storage = DynamoStorage::new(table_name).await;

                            let previous_leaderboard =
                                storage.load_previous(YEAR, LEADERBOARD_ID).await;
                            assert_matches!(
                                previous_leaderboard,
                                Err(aoc_leaderbot_lib::Error::Aws(
                                    AwsError::Dynamo(
                                        DynamoError::LoadPreviousLeaderboard {
                                            leaderboard_id,
                                            year,
                                            source: LoadPreviousDynamoError::ParseError(_),
                                        }
                                    )
                                )) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                            );
                        })
                    };

                    run_test(test).await;
                }
            }
        }

        mod save {
            use super::*;

            #[tokio::test]
            async fn without_existing() {
                let test = |table_name: String| {
                    AssertUnwindSafe(async {
                        let mut storage = DynamoStorage::new(table_name.clone()).await;

                        let expected_leaderboard = get_sample_leaderboard();
                        storage
                            .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                            .await
                            .unwrap();

                        let actual_leaderboard = load_leaderboard(table_name).await;
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    })
                };

                run_test(test).await;
            }

            #[tokio::test]
            async fn with_existing() {
                let test = |table_name: String| {
                    AssertUnwindSafe(async {
                        let previous_leaderboard = get_sample_leaderboard();
                        save_leaderboard(table_name.clone(), &previous_leaderboard).await;

                        let mut storage = DynamoStorage::new(table_name.clone()).await;

                        let expected_leaderboard = Leaderboard {
                            day1_ts: previous_leaderboard.day1_ts + 1,
                            ..previous_leaderboard
                        };
                        storage
                            .save(YEAR, LEADERBOARD_ID, &expected_leaderboard)
                            .await
                            .unwrap();

                        let actual_leaderboard = load_leaderboard(table_name).await;
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    })
                };

                run_test(test).await;
            }

            mod errors {
                use super::*;

                #[tokio::test]
                async fn put_item() {
                    let table_name = random_table_name();
                    let mut storage = DynamoStorage::new(table_name).await;

                    let leaderboard = get_sample_leaderboard();
                    let save_result = storage.save(YEAR, LEADERBOARD_ID, &leaderboard).await;
                    assert_matches!(
                        save_result,
                        Err(aoc_leaderbot_lib::Error::Aws(
                            AwsError::Dynamo(
                                DynamoError::SaveLeaderboard {
                                    leaderboard_id,
                                    year,
                                    source: SaveDynamoError::PutItem(_),
                                }
                            ))
                        ) if leaderboard_id == LEADERBOARD_ID && year == YEAR
                    );
                }
            }
        }

        mod create_table {
            use super::*;

            #[tokio::test]
            async fn create_table() {
                let test = |table_name: String| {
                    AssertUnwindSafe(async move {
                        let storage = DynamoStorage::new(table_name.clone()).await;

                        let create_result = storage.create_table().await;
                        assert_matches!(
                            create_result,
                            Err(aoc_leaderbot_lib::Error::Aws(
                                AwsError::Dynamo(
                                    DynamoError::CreateTable {
                                        table_name: actual_table_name,
                                        source: CreateDynamoTableError::CreateTable(_),
                                    }
                                ))
                            ) if actual_table_name == table_name
                        );
                    })
                };

                run_test(test).await;
            }
        }
    }
}
