// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod dynamo_storage {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderboard::test_helpers::{test_leaderboard, TEST_LEADERBOARD_ID, TEST_YEAR};
    use aoc_leaderbot_aws_lib::error::{
        CreateDynamoDbTableError, DynamoDbError, LoadPreviousDynamoDbError, SaveDynamoDbError,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::{
        local_non_existent_table, LocalTable,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::{
        HASH_KEY, LEADERBOARD_DATA, RANGE_KEY,
    };
    use aoc_leaderbot_lib::leaderbot::Storage;
    use aoc_leaderbot_lib::ErrorKind;
    use assert_matches::assert_matches;
    use aws_sdk_dynamodb::types::AttributeValue;
    use rstest::rstest;

    mod storage_impl {
        use super::*;

        pub mod load_previous {
            use super::*;

            #[test_log::test]
            fn without_existing() {
                LocalTable::run_test(|mut table| async move {
                    let previous_leaderboard = table
                        .storage()
                        .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                        .await;
                    assert_matches!(previous_leaderboard, Ok((None, None)));
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard(
                #[from(test_leaderboard)] expected_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&expected_leaderboard).await;

                    let previous = table
                        .storage()
                        .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                        .await;
                    assert_matches!(previous, Ok((Some(actual_leaderboard), None)) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    });
                });
            }

            #[test_log::test]
            fn with_existing_last_error() {
                LocalTable::run_test(|mut table| async move {
                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;

                    let previous = table
                        .storage()
                        .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                        .await;
                    assert_matches!(previous, Ok((None, Some(actual_last_error))) => {
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error
                        );
                    });
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard_and_last_error(
                #[from(test_leaderboard)] expected_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&expected_leaderboard).await;
                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;

                    let previous = table
                        .storage()
                        .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                        .await;
                    assert_matches!(previous, Ok((Some(actual_leaderboard), Some(actual_last_error))) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error
                        );
                    });
                });
            }

            pub mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn get_item(
                    #[future]
                    #[from(local_non_existent_table)]
                    table: LocalTable,
                ) {
                    let mut table = table;
                    let previous_leaderboard = table
                        .storage()
                        .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                        .await;
                    assert_matches!(
                        previous_leaderboard,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::LoadPreviousLeaderboard {
                                leaderboard_id,
                                year,
                                source: LoadPreviousDynamoDbError::GetItem(_),
                            }
                        )) => {
                            assert_eq!(TEST_LEADERBOARD_ID, leaderboard_id);
                            assert_eq!(TEST_YEAR, year);
                        }
                    );
                }

                #[test_log::test]
                fn invalid_leaderboard_data_type() {
                    LocalTable::run_test(|mut table| async move {
                        table
                            .client()
                            .put_item()
                            .table_name(table.name())
                            .item(HASH_KEY, AttributeValue::N(TEST_LEADERBOARD_ID.to_string()))
                            .item(RANGE_KEY, AttributeValue::N(TEST_YEAR.to_string()))
                            .item(LEADERBOARD_DATA, AttributeValue::N(42.to_string()))
                            .send()
                            .await
                            .unwrap();

                        let previous_leaderboard = table
                            .storage()
                            .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                            .await;
                        assert_matches!(
                            previous_leaderboard,
                            Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                                DynamoDbError::LoadPreviousLeaderboard {
                                    leaderboard_id,
                                    year,
                                    source: LoadPreviousDynamoDbError::Deserialize(_),
                                }
                            )) => {
                                assert_eq!(TEST_LEADERBOARD_ID, leaderboard_id);
                                assert_eq!(TEST_YEAR, year);
                            }
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
                            .item(HASH_KEY, AttributeValue::N(TEST_LEADERBOARD_ID.to_string()))
                            .item(RANGE_KEY, AttributeValue::N(TEST_YEAR.to_string()))
                            .item(
                                LEADERBOARD_DATA,
                                AttributeValue::S("{\"hello\":\"world\"".to_string()),
                            )
                            .send()
                            .await
                            .unwrap();

                        let previous_leaderboard = table
                            .storage()
                            .load_previous(TEST_YEAR, TEST_LEADERBOARD_ID)
                            .await;
                        assert_matches!(
                            previous_leaderboard,
                            Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                                DynamoDbError::LoadPreviousLeaderboard {
                                    leaderboard_id,
                                    year,
                                    source: LoadPreviousDynamoDbError::Deserialize(_),
                                }
                            )) => {
                                assert_eq!(TEST_LEADERBOARD_ID, leaderboard_id);
                                assert_eq!(TEST_YEAR, year);
                            }
                        );
                    });
                }
            }
        }

        pub mod save_success {
            use super::*;

            #[rstest]
            #[test_log::test]
            fn without_existing(#[from(test_leaderboard)] expected_leaderboard: Leaderboard) {
                LocalTable::run_test(|mut table| async move {
                    table
                        .storage()
                        .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &expected_leaderboard)
                        .await
                        .unwrap();

                    let (actual_leaderboard, actual_last_error) =
                        table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual_leaderboard, Some(actual_leaderboard) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    });
                    assert!(actual_last_error.is_none());
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard(
                #[from(test_leaderboard)] previous_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&previous_leaderboard).await;

                    let expected_leaderboard = Leaderboard {
                        day1_ts: previous_leaderboard.day1_ts + 1,
                        ..previous_leaderboard
                    };
                    table
                        .storage()
                        .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &expected_leaderboard)
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    });
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_last_error(
                #[from(test_leaderboard)] expected_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;

                    table
                        .storage()
                        .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &expected_leaderboard)
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    });
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard_and_last_error(
                #[from(test_leaderboard)] previous_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&previous_leaderboard).await;
                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;

                    let expected_leaderboard = Leaderboard {
                        day1_ts: previous_leaderboard.day1_ts + 1,
                        ..previous_leaderboard
                    };
                    table
                        .storage()
                        .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &expected_leaderboard)
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                    });
                });
            }

            pub mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn put_item(
                    #[future]
                    #[from(local_non_existent_table)]
                    table: LocalTable,
                    #[from(test_leaderboard)] leaderboard: Leaderboard,
                ) {
                    let mut table = table;
                    let save_result = table
                        .storage()
                        .save_success(TEST_YEAR, TEST_LEADERBOARD_ID, &leaderboard)
                        .await;
                    assert_matches!(
                        save_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::SaveLeaderboard {
                                leaderboard_id,
                                year,
                                source: SaveDynamoDbError::PutItem(_),
                            }
                        )) => {
                            assert_eq!(TEST_LEADERBOARD_ID, leaderboard_id);
                            assert_eq!(TEST_YEAR, year);
                        }
                    );
                }
            }
        }

        pub mod save_error {
            use super::*;

            #[test_log::test]
            fn without_existing() {
                LocalTable::run_test(|mut table| async move {
                    table
                        .storage()
                        .save_error(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                        )
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (None, Some(actual_last_error)) => {
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error,
                        );
                    });
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard(
                #[from(test_leaderboard)] expected_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&expected_leaderboard).await;

                    table
                        .storage()
                        .save_error(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                        )
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), Some(actual_last_error)) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error,
                        );
                    });
                });
            }

            #[test_log::test]
            fn with_existing_last_error() {
                LocalTable::run_test(|mut table| async move {
                    table.save_last_error(ErrorKind::MissingField).await;

                    table
                        .storage()
                        .save_error(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                        )
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (None, Some(actual_last_error)) => {
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error,
                        );
                    });
                });
            }

            #[rstest]
            #[test_log::test]
            fn with_existing_leaderboard_and_last_error(
                #[from(test_leaderboard)] expected_leaderboard: Leaderboard,
            ) {
                LocalTable::run_test(|mut table| async move {
                    table.save_leaderboard(&expected_leaderboard).await;
                    table.save_last_error(ErrorKind::MissingField).await;

                    table
                        .storage()
                        .save_error(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                        )
                        .await
                        .unwrap();

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), Some(actual_last_error)) => {
                        assert_eq!(expected_leaderboard, actual_leaderboard);
                        assert_eq!(
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                            actual_last_error,
                        );
                    });
                });
            }

            pub mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[test_log::test(tokio::test)]
                async fn update_item(
                    #[future]
                    #[from(local_non_existent_table)]
                    table: LocalTable,
                ) {
                    let mut table = table;
                    let save_result = table
                        .storage()
                        .save_error(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            ErrorKind::Leaderboard(aoc_leaderboard::ErrorKind::NoAccess),
                        )
                        .await;
                    assert_matches!(
                        save_result,
                        Err(aoc_leaderbot_aws_lib::Error::Dynamo(
                            DynamoDbError::SaveLastError {
                                leaderboard_id,
                                year,
                                source: SaveDynamoDbError::UpdateItem(_),
                            }
                        )) => {
                            assert_eq!(TEST_LEADERBOARD_ID, leaderboard_id);
                            assert_eq!(TEST_YEAR, year);
                        }
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
