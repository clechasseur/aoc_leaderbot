#[cfg(all(
    any(not(ci), target_os = "linux"), // These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
    feature = "__testing"              // These tests only work if you compile with the internal `__testing` feature.
))]
mod bot_lambda_handler {
    use std::collections::HashMap;
    use std::future::Future;

    use aoc_leaderboard::aoc::{
        CompletionDayLevel, Leaderboard, LeaderboardCredentials, LeaderboardMember,
        PuzzleCompletionInfo,
    };
    use aoc_leaderboard::reqwest::Method;
    use aoc_leaderboard::test_helpers::{
        TEST_AOC_SESSION, TEST_DAY_1_TS, TEST_DAY_2_TS, TEST_LEADERBOARD_ID, TEST_YEAR,
        mock_server_with_leaderboard, test_leaderboard_credentials,
    };
    use aoc_leaderboard::wiremock::matchers::{header, method, path};
    use aoc_leaderboard::wiremock::{Mock, MockServer, ResponseTemplate};
    use aoc_leaderbot_aws_lambda_impl::leaderbot::{
        IncomingDynamoDbStorageInput, IncomingMessage, IncomingSlackWebhookReporterInput,
        OutgoingMessage, bot_lambda_handler,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::test_helpers::{
        LOCAL_ENDPOINT_URL, LocalTable,
    };
    use aoc_leaderbot_lib::ErrorKind;
    use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::LeaderboardSortOrder;
    use assert_matches::assert_matches;
    use lambda_runtime::{Context, LambdaEvent};
    use rstest::{fixture, rstest};
    use serial_test::serial;

    const OWNER: u64 = 42;
    const MEMBER_1: u64 = 23;
    const MEMBER_2: u64 = 11;

    const WEBHOOK_PATH: &str = "/webhook";
    const CHANNEL: &str = "#aoc_leaderbot_test";
    const USERNAME: &str = "AoC Leaderbot (test)";
    const ICON_URL: &str = "https://www.adventofcode.com/favicon.ico";

    #[fixture]
    fn base_leaderboard() -> Leaderboard {
        Leaderboard {
            year: TEST_YEAR,
            owner_id: OWNER,
            day1_ts: *TEST_DAY_1_TS,
            members: {
                let mut members = HashMap::new();

                members.insert(
                    OWNER,
                    LeaderboardMember {
                        name: Some("clechasseur".to_string()),
                        id: OWNER,
                        stars: 0,
                        local_score: 0,
                        global_score: 0,
                        last_star_ts: 0,
                        completion_day_level: HashMap::new(),
                    },
                );
                members.insert(
                    MEMBER_1,
                    LeaderboardMember {
                        name: None,
                        id: MEMBER_1,
                        stars: 2,
                        local_score: 10,
                        global_score: 0,
                        last_star_ts: *TEST_DAY_1_TS,
                        completion_day_level: {
                            let mut completion_day_level = HashMap::new();

                            completion_day_level.insert(
                                1,
                                CompletionDayLevel {
                                    part_1: PuzzleCompletionInfo {
                                        get_star_ts: *TEST_DAY_1_TS,
                                        star_index: 1,
                                    },
                                    part_2: Some(PuzzleCompletionInfo {
                                        get_star_ts: *TEST_DAY_1_TS,
                                        star_index: 2,
                                    }),
                                },
                            );

                            completion_day_level
                        },
                    },
                );

                members
            },
        }
    }

    #[fixture]
    fn leaderboard_with_new_member() -> Leaderboard {
        let mut leaderboard = base_leaderboard();

        leaderboard.members.insert(
            MEMBER_2,
            LeaderboardMember {
                name: None,
                id: MEMBER_2,
                stars: 1,
                local_score: 2,
                global_score: 0,
                last_star_ts: *TEST_DAY_2_TS,
                completion_day_level: {
                    let mut completion_day_level = HashMap::new();

                    completion_day_level.insert(
                        1,
                        CompletionDayLevel {
                            part_1: PuzzleCompletionInfo {
                                get_star_ts: *TEST_DAY_2_TS,
                                star_index: 1,
                            },
                            part_2: None,
                        },
                    );

                    completion_day_level
                },
            },
        );

        leaderboard
    }

    async fn mount_slack_webhook_handler(mock_server: &MockServer, expect_report: bool) {
        Mock::given(method(Method::POST))
            .and(path(WEBHOOK_PATH))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .up_to_n_times(1)
            .expect(if expect_report { 1 } else { 0 })
            .named("slack-webhook-handler")
            .mount(mock_server)
            .await;
    }

    fn run_bot_test<TF, TFR>(leaderboard: Leaderboard, expect_report: bool, test_f: TF)
    where
        TF: FnOnce(MockServer, LocalTable) -> TFR + Send + 'static,
        TFR: Future<Output = ()> + Send + 'static,
    {
        LocalTable::run_test(|table| async move {
            let mock_server =
                mock_server_with_leaderboard(leaderboard, test_leaderboard_credentials::default())
                    .await;
            mount_slack_webhook_handler(&mock_server, expect_report).await;

            test_f(mock_server, table).await;
        });
    }

    mod using_event {
        use super::*;

        fn incoming_message(
            test_run: bool,
            mock_server: &MockServer,
            table: &LocalTable,
        ) -> IncomingMessage {
            IncomingMessage {
                year: Some(TEST_YEAR),
                leaderboard_id: Some(TEST_LEADERBOARD_ID),
                credentials: Some(LeaderboardCredentials::SessionCookie(TEST_AOC_SESSION.into())),
                test_run,
                aoc_base_url: Some(mock_server.uri()),
                dynamodb_storage_input: IncomingDynamoDbStorageInput {
                    table_name: Some(table.name().into()),
                    test_endpoint_url: Some(LOCAL_ENDPOINT_URL.into()),
                    test_region: Some("ca-central-1".into()),
                },
                slack_webhook_reporter_input: IncomingSlackWebhookReporterInput {
                    webhook_url: Some(format!("{}{WEBHOOK_PATH}", mock_server.uri())),
                    channel: Some(CHANNEL.into()),
                    username: Some(USERNAME.into()),
                    icon_url: Some(ICON_URL.into()),
                    sort_order: Some(LeaderboardSortOrder::Stars),
                },
            }
        }

        mod without_previous {
            use super::*;

            #[rstest]
            #[test_log::test]
            fn stores_current(#[from(base_leaderboard)] current_leaderboard: Leaderboard) {
                run_bot_test(current_leaderboard.clone(), false, |mock_server, table| async move {
                    let incoming_message = incoming_message(false, &mock_server, &table);
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Ok(OutgoingMessage { output }) => {
                        assert_eq!(output.year, TEST_YEAR);
                        assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                        assert!(output.previous_leaderboard.is_none());
                        assert_eq!(output.leaderboard, current_leaderboard);
                        assert!(output.changes.is_none());
                    });

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(current_leaderboard, actual_leaderboard);
                    });
                });
            }

            mod test_run {
                use super::*;

                #[rstest]
                #[test_log::test]
                fn does_not_store_current_and_sends_test_report(
                    #[from(base_leaderboard)] current_leaderboard: Leaderboard,
                ) {
                    run_bot_test(
                        current_leaderboard.clone(),
                        true,
                        |mock_server, table| async move {
                            let incoming_message = incoming_message(true, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, TEST_YEAR);
                                assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                assert!(output.previous_leaderboard.is_none());
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert!(output.changes.is_none());
                            });

                            let actual = table.load_leaderboard_and_last_error().await;
                            assert_matches!(actual, (None, None));
                        },
                    );
                }
            }
        }

        mod with_previous_leaderboard {
            use super::*;

            mod without_changes {
                use super::*;

                #[rstest]
                #[test_log::test]
                fn does_not_send_report(
                    #[from(base_leaderboard)] current_leaderboard: Leaderboard,
                ) {
                    run_bot_test(
                        current_leaderboard.clone(),
                        false,
                        |mock_server, table| async move {
                            table.save_leaderboard(&current_leaderboard).await;

                            let incoming_message = incoming_message(false, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, TEST_YEAR);
                                assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                assert_matches!(output.previous_leaderboard, Some(leaderboard) => {
                                    assert_eq!(current_leaderboard, leaderboard);
                                });
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert!(output.changes.is_none());
                            });

                            let actual = table.load_leaderboard_and_last_error().await;
                            assert_matches!(actual, (Some(actual_leaderboard), None) => {
                                assert_eq!(current_leaderboard, actual_leaderboard);
                            });
                        },
                    );
                }

                mod test_run {
                    use super::*;

                    #[rstest]
                    #[test_log::test]
                    fn sends_test_report(
                        #[from(base_leaderboard)] current_leaderboard: Leaderboard,
                    ) {
                        run_bot_test(
                            current_leaderboard.clone(),
                            true,
                            |mock_server, table| async move {
                                table.save_leaderboard(&current_leaderboard).await;

                                let incoming_message = incoming_message(true, &mock_server, &table);
                                let event = LambdaEvent::new(incoming_message, Context::default());
                                let result = bot_lambda_handler(event).await;

                                assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                    assert_eq!(output.year, TEST_YEAR);
                                    assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                    assert_matches!(output.previous_leaderboard, Some(leaderboard) => {
                                        assert_eq!(current_leaderboard, leaderboard);
                                    });
                                    assert_eq!(output.leaderboard, current_leaderboard);
                                    assert!(output.changes.is_none());
                                });

                                let actual = table.load_leaderboard_and_last_error().await;
                                assert_matches!(actual, (Some(actual_leaderboard), None) => {
                                    assert_eq!(current_leaderboard, actual_leaderboard);
                                });
                            },
                        );
                    }
                }
            }

            mod with_changes {
                use super::*;

                #[rstest]
                #[test_log::test]
                fn updates_storage_and_sends_report(
                    #[from(base_leaderboard)] previous_leaderboard: Leaderboard,
                    #[from(leaderboard_with_new_member)] current_leaderboard: Leaderboard,
                ) {
                    run_bot_test(
                        current_leaderboard.clone(),
                        true,
                        |mock_server, table| async move {
                            table.save_leaderboard(&previous_leaderboard).await;

                            let incoming_message = incoming_message(false, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, TEST_YEAR);
                                assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                assert_matches!(output.previous_leaderboard, Some(leaderboard) => {
                                    assert_eq!(previous_leaderboard, leaderboard);
                                });
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert_matches!(output.changes, Some(changes) => {
                                    assert_eq!(changes.new_members, [MEMBER_2].into());
                                    assert!(changes.members_with_new_stars.is_empty());
                                });
                            });

                            let actual = table.load_leaderboard_and_last_error().await;
                            assert_matches!(actual, (Some(actual_leaderboard), None) => {
                                assert_eq!(current_leaderboard, actual_leaderboard);
                            });
                        },
                    );
                }

                mod test_run {
                    use super::*;

                    #[rstest]
                    #[test_log::test]
                    fn does_not_update_storage_and_sends_test_report(
                        #[from(base_leaderboard)] previous_leaderboard: Leaderboard,
                        #[from(leaderboard_with_new_member)] current_leaderboard: Leaderboard,
                    ) {
                        run_bot_test(
                            current_leaderboard.clone(),
                            true,
                            |mock_server, table| async move {
                                table.save_leaderboard(&previous_leaderboard).await;

                                let incoming_message = incoming_message(true, &mock_server, &table);
                                let event = LambdaEvent::new(incoming_message, Context::default());
                                let result = bot_lambda_handler(event).await;

                                assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                    assert_eq!(output.year, TEST_YEAR);
                                    assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                    assert_matches!(output.previous_leaderboard, Some(leaderboard) => {
                                        assert_eq!(previous_leaderboard, leaderboard);
                                    });
                                    assert_eq!(output.leaderboard, current_leaderboard);
                                    assert_matches!(output.changes, Some(changes) => {
                                        assert_eq!(changes.new_members, [MEMBER_2].into());
                                        assert!(changes.members_with_new_stars.is_empty());
                                    });
                                });

                                let actual = table.load_leaderboard_and_last_error().await;
                                assert_matches!(actual, (Some(actual_leaderboard), None) => {
                                    assert_eq!(previous_leaderboard, actual_leaderboard);
                                });
                            },
                        );
                    }
                }
            }
        }

        mod with_previous_last_error {
            use super::*;

            #[rstest]
            #[test_log::test]
            fn overwrites_last_error(#[from(base_leaderboard)] current_leaderboard: Leaderboard) {
                run_bot_test(current_leaderboard.clone(), false, |mock_server, table| async move {
                    table
                        .save_last_error(ErrorKind::Leaderboard(
                            aoc_leaderboard::ErrorKind::NoAccess,
                        ))
                        .await;

                    let incoming_message = incoming_message(false, &mock_server, &table);
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Ok(OutgoingMessage { output }) => {
                        assert_eq!(output.year, TEST_YEAR);
                        assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                        assert!(output.previous_leaderboard.is_none());
                        assert_eq!(output.leaderboard, current_leaderboard);
                        assert!(output.changes.is_none());
                    });

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(current_leaderboard, actual_leaderboard);
                    });
                });
            }

            mod test_run {
                use super::*;

                #[rstest]
                #[test_log::test]
                fn does_not_overwrite_last_error(
                    #[from(base_leaderboard)] current_leaderboard: Leaderboard,
                ) {
                    run_bot_test(
                        current_leaderboard.clone(),
                        true,
                        |mock_server, table| async move {
                            table
                                .save_last_error(ErrorKind::Leaderboard(
                                    aoc_leaderboard::ErrorKind::NoAccess,
                                ))
                                .await;

                            let incoming_message = incoming_message(true, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, TEST_YEAR);
                                assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                                assert!(output.previous_leaderboard.is_none());
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert!(output.changes.is_none());
                            });

                            let actual = table.load_leaderboard_and_last_error().await;
                            assert_matches!(
                                actual,
                                (
                                    None,
                                    Some(ErrorKind::Leaderboard(
                                        aoc_leaderboard::ErrorKind::NoAccess
                                    ))
                                )
                            );
                        },
                    );
                }
            }
        }
    }

    mod using_environment {
        use std::env;

        use aoc_leaderbot_aws_lambda_impl::leaderbot::CONFIG_ENV_VAR_PREFIX;
        use aoc_leaderbot_lib::error::EnvVarError;
        use aoc_leaderbot_lib::leaderbot::config::env::{
            ENV_CONFIG_LEADERBOARD_ID_SUFFIX, ENV_CONFIG_SESSION_COOKIE_SUFFIX,
            ENV_CONFIG_YEAR_SUFFIX,
        };
        use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
            CHANNEL_ENV_VAR, WEBHOOK_URL_ENV_VAR,
        };

        use super::*;

        fn configure_environment(mock_server: &MockServer) {
            unsafe {
                let set_env_config_var = |suffix, value: &str| {
                    env::set_var(format!("{CONFIG_ENV_VAR_PREFIX}{suffix}"), value)
                };

                set_env_config_var(ENV_CONFIG_YEAR_SUFFIX, &TEST_YEAR.to_string());
                set_env_config_var(
                    ENV_CONFIG_LEADERBOARD_ID_SUFFIX,
                    &TEST_LEADERBOARD_ID.to_string(),
                );
                set_env_config_var(ENV_CONFIG_SESSION_COOKIE_SUFFIX, TEST_AOC_SESSION);

                env::set_var(WEBHOOK_URL_ENV_VAR, format!("{}{WEBHOOK_PATH}", mock_server.uri()));
                env::set_var(CHANNEL_ENV_VAR, CHANNEL);
            }
        }

        fn incoming_message(mock_server: &MockServer, table: &LocalTable) -> IncomingMessage {
            IncomingMessage {
                year: None,
                leaderboard_id: None,
                credentials: None,
                test_run: false,
                aoc_base_url: Some(mock_server.uri()),
                dynamodb_storage_input: IncomingDynamoDbStorageInput {
                    table_name: Some(table.name().into()),
                    test_endpoint_url: Some(LOCAL_ENDPOINT_URL.into()),
                    test_region: Some("ca-central-1".into()),
                },
                slack_webhook_reporter_input: IncomingSlackWebhookReporterInput::default(),
            }
        }

        mod with_changes {
            use super::*;

            #[rstest]
            #[test_log::test]
            #[serial(aws_lambda_env)]
            fn updates_storage_and_sends_report(
                #[from(base_leaderboard)] previous_leaderboard: Leaderboard,
                #[from(leaderboard_with_new_member)] current_leaderboard: Leaderboard,
            ) {
                run_bot_test(current_leaderboard.clone(), true, |mock_server, table| async move {
                    table.save_leaderboard(&previous_leaderboard).await;
                    configure_environment(&mock_server);

                    let incoming_message = incoming_message(&mock_server, &table);
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Ok(OutgoingMessage { output }) => {
                        assert_eq!(output.year, TEST_YEAR);
                        assert_eq!(output.leaderboard_id, TEST_LEADERBOARD_ID);
                        assert_matches!(
                            output.previous_leaderboard,
                            Some(leaderboard) if leaderboard == previous_leaderboard
                        );
                        assert_eq!(output.leaderboard, current_leaderboard);
                        assert_matches!(output.changes, Some(changes) => {
                            assert_eq!(changes.new_members, [MEMBER_2].into());
                            assert!(changes.members_with_new_stars.is_empty());
                        });
                    });

                    let actual = table.load_leaderboard_and_last_error().await;
                    assert_matches!(actual, (Some(actual_leaderboard), None) => {
                        assert_eq!(current_leaderboard, actual_leaderboard);
                    });
                });
            }
        }

        mod errors {
            use super::*;

            mod config {
                use super::*;

                #[test_log::test(tokio::test)]
                #[serial(aws_lambda_env)]
                async fn invalid_year_env_var() {
                    unsafe {
                        env::set_var(
                            format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_YEAR_SUFFIX}"),
                            "invalid",
                        );
                        env::set_var(
                            format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_LEADERBOARD_ID_SUFFIX}"),
                            TEST_LEADERBOARD_ID.to_string(),
                        );
                        env::set_var(
                            format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_SESSION_COOKIE_SUFFIX}"),
                            TEST_AOC_SESSION,
                        );
                    }

                    let incoming_message = IncomingMessage::default();
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Err(lambda_err) => {
                        assert_matches!(lambda_err.downcast::<aoc_leaderbot_lib::Error>(), Ok(err) => {
                            assert_matches!(*err, aoc_leaderbot_lib::Error::Env { source: env_err, .. } => {
                                assert_matches!(env_err, EnvVarError::IntExpected { .. });
                            });
                        });
                    });
                }
            }

            mod reporter {
                use aoc_leaderbot_slack_lib::error::WebhookError;
                use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::SlackWebhookReporterBuilderError;

                use super::*;

                #[test_log::test(tokio::test)]
                #[serial(aws_lambda_env)]
                async fn missing_slack_reporter_env_vars() {
                    unsafe {
                        env::remove_var(format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_YEAR_SUFFIX}"));
                        env::set_var(
                            format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_LEADERBOARD_ID_SUFFIX}"),
                            TEST_LEADERBOARD_ID.to_string(),
                        );
                        env::set_var(
                            format!("{CONFIG_ENV_VAR_PREFIX}{ENV_CONFIG_SESSION_COOKIE_SUFFIX}"),
                            TEST_AOC_SESSION,
                        );

                        env::remove_var(WEBHOOK_URL_ENV_VAR);
                        env::remove_var(CHANNEL_ENV_VAR);
                    }

                    let incoming_message = IncomingMessage::default();
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Err(lambda_err) => {
                        assert_matches!(lambda_err.downcast::<aoc_leaderbot_slack_lib::Error>(), Ok(err) => {
                            assert_matches!(*err, aoc_leaderbot_slack_lib::Error::Webhook(
                                WebhookError::ReporterBuilder(SlackWebhookReporterBuilderError::ValidationError(_))
                            ));
                        });
                    });
                }
            }
        }
    }
}
