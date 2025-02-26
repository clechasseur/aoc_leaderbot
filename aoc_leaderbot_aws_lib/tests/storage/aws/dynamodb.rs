// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod dynamo_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_aws_lib::error::{
        CreateDynamoDbTableError, DynamoDbError, LoadPreviousDynamoDbError, SaveDynamoDbError,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::testing::LocalTable;
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::{
        HASH_KEY, LEADERBOARD_DATA, RANGE_KEY,
    };
    use aoc_leaderbot_lib::leaderbot::Storage;
    use aoc_leaderbot_test_helpers::{get_sample_leaderboard, LEADERBOARD_ID, YEAR};
    use assert_matches::assert_matches;
    use aws_sdk_dynamodb::types::AttributeValue;

    mod storage_impl {
        use super::*;

        pub mod load_previous {
            use super::*;

            #[test_log::test]
            fn without_data() {
                LocalTable::run_test(|mut table| async move {
                    let previous_leaderboard =
                        table.storage().load_previous(YEAR, LEADERBOARD_ID).await;
                    assert_matches!(previous_leaderboard, Ok(None));
                });
            }

            #[test_log::test]
            fn with_data() {
                LocalTable::run_test(|mut table| async move {
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
                    LocalTable::run_test(|mut table| async move {
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
                    LocalTable::run_test(|mut table| async move {
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
                    LocalTable::run_test(|mut table| async move {
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
                LocalTable::run_test(|mut table| async move {
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
                LocalTable::run_test(|mut table| async move {
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
                    LocalTable::run_test(|mut table| async move {
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
