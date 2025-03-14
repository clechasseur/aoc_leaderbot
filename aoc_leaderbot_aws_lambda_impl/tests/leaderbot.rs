#[cfg(all(
    any(not(ci), target_os = "linux"), // These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
    feature = "__testing"              // These tests only work if you compile with the internal `__testing` feature.
))]
mod bot_lambda_handler {
    use std::collections::HashMap;
    use std::future::Future;

    use aoc_leaderboard::aoc::{
        CompletionDayLevel, Leaderboard, LeaderboardMember, PuzzleCompletionInfo,
    };
    use aoc_leaderboard::reqwest::Method;
    use aoc_leaderbot_aws_lambda_impl::leaderbot::{
        bot_lambda_handler, IncomingDynamoDbStorageInput, IncomingMessage,
        IncomingSlackWebhookReporterInput, OutgoingMessage,
    };
    use aoc_leaderbot_aws_lib::leaderbot::storage::aws::dynamodb::testing::{
        LocalTable, LOCAL_ENDPOINT_URL,
    };
    use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::LeaderboardSortOrder;
    use aoc_leaderbot_test_helpers::{
        get_mock_server_with_leaderboard, AOC_SESSION, LEADERBOARD_ID, YEAR,
    };
    use assert_matches::assert_matches;
    use chrono::Local;
    use lambda_runtime::{Context, LambdaEvent};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const OWNER: u64 = 42;
    const MEMBER_1: u64 = 23;
    const MEMBER_2: u64 = 11;

    const WEBHOOK_PATH: &str = "/webhook";
    const CHANNEL: &str = "#aoc_leaderbot_test";
    const USERNAME: &str = "AoC Leaderbot (test)";
    const ICON_URL: &str = "https://www.adventofcode.com/favicon.ico";

    fn base_leaderboard() -> Leaderboard {
        Leaderboard {
            year: YEAR,
            owner_id: OWNER,
            day1_ts: Local::now().timestamp(),
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
                        last_star_ts: Local::now().timestamp(),
                        completion_day_level: {
                            let mut completion_day_level = HashMap::new();

                            completion_day_level.insert(
                                1,
                                CompletionDayLevel {
                                    part_1: PuzzleCompletionInfo {
                                        get_star_ts: Local::now().timestamp(),
                                        star_index: 1,
                                    },
                                    part_2: Some(PuzzleCompletionInfo {
                                        get_star_ts: Local::now().timestamp(),
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
                last_star_ts: Local::now().timestamp(),
                completion_day_level: {
                    let mut completion_day_level = HashMap::new();

                    completion_day_level.insert(
                        1,
                        CompletionDayLevel {
                            part_1: PuzzleCompletionInfo {
                                get_star_ts: Local::now().timestamp(),
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
            let mock_server = get_mock_server_with_leaderboard(leaderboard).await;
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
                year: Some(YEAR),
                leaderboard_id: Some(LEADERBOARD_ID),
                aoc_session: Some(AOC_SESSION.into()),
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

            #[test_log::test]
            fn stores_current() {
                let current_leaderboard = base_leaderboard();
                run_bot_test(current_leaderboard.clone(), false, |mock_server, table| async move {
                    let incoming_message = incoming_message(false, &mock_server, &table);
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Ok(OutgoingMessage { output }) => {
                        assert_eq!(output.year, YEAR);
                        assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
                        assert!(output.previous_leaderboard.is_none());
                        assert_eq!(output.leaderboard, current_leaderboard);
                        assert!(output.changes.is_none());
                    });
                    assert_eq!(table.load_leaderboard().await, current_leaderboard);
                });
            }

            mod test_run {
                use aoc_leaderbot_lib::leaderbot::Storage;

                use super::*;

                #[test_log::test]
                fn does_not_store_current_and_sends_test_report() {
                    let current_leaderboard = base_leaderboard();
                    run_bot_test(
                        current_leaderboard.clone(),
                        true,
                        |mock_server, mut table| async move {
                            let incoming_message = incoming_message(true, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, YEAR);
                                assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
                                assert!(output.previous_leaderboard.is_none());
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert!(output.changes.is_none());
                            });
                            assert_matches!(
                                table.storage().load_previous(YEAR, LEADERBOARD_ID).await,
                                Ok(None)
                            );
                        },
                    );
                }
            }
        }

        mod with_previous {
            use super::*;

            mod without_changes {
                use super::*;

                #[test_log::test]
                fn does_not_send_report() {
                    let current_leaderboard = base_leaderboard();
                    run_bot_test(
                        current_leaderboard.clone(),
                        false,
                        |mock_server, table| async move {
                            table.save_leaderboard(&current_leaderboard).await;

                            let incoming_message = incoming_message(false, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, YEAR);
                                assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
                                assert_matches!(
                                    output.previous_leaderboard,
                                    Some(leaderboard) if leaderboard == current_leaderboard
                                );
                                assert_eq!(output.leaderboard, current_leaderboard);
                                assert!(output.changes.is_none());
                            });
                            assert_eq!(table.load_leaderboard().await, current_leaderboard);
                        },
                    );
                }

                mod test_run {
                    use super::*;

                    #[test_log::test]
                    fn sends_test_report() {
                        let current_leaderboard = base_leaderboard();
                        run_bot_test(
                            current_leaderboard.clone(),
                            true,
                            |mock_server, table| async move {
                                table.save_leaderboard(&current_leaderboard).await;

                                let incoming_message = incoming_message(true, &mock_server, &table);
                                let event = LambdaEvent::new(incoming_message, Context::default());
                                let result = bot_lambda_handler(event).await;

                                assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                    assert_eq!(output.year, YEAR);
                                    assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
                                    assert_matches!(
                                        output.previous_leaderboard,
                                        Some(leaderboard) if leaderboard == current_leaderboard
                                    );
                                    assert_eq!(output.leaderboard, current_leaderboard);
                                    assert!(output.changes.is_none());
                                });
                                assert_eq!(table.load_leaderboard().await, current_leaderboard);
                            },
                        );
                    }
                }
            }

            mod with_changes {
                use super::*;

                #[test_log::test]
                fn updates_storage_and_sends_report() {
                    let previous_leaderboard = base_leaderboard();
                    let current_leaderboard = leaderboard_with_new_member();
                    run_bot_test(
                        current_leaderboard.clone(),
                        true,
                        |mock_server, table| async move {
                            table.save_leaderboard(&previous_leaderboard).await;

                            let incoming_message = incoming_message(false, &mock_server, &table);
                            let event = LambdaEvent::new(incoming_message, Context::default());
                            let result = bot_lambda_handler(event).await;

                            assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                assert_eq!(output.year, YEAR);
                                assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
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
                            assert_eq!(table.load_leaderboard().await, current_leaderboard);
                        },
                    );
                }

                mod test_run {
                    use super::*;

                    #[test_log::test]
                    fn does_not_update_storage_and_sends_test_report() {
                        let previous_leaderboard = base_leaderboard();
                        let current_leaderboard = leaderboard_with_new_member();
                        run_bot_test(
                            current_leaderboard.clone(),
                            true,
                            |mock_server, table| async move {
                                table.save_leaderboard(&previous_leaderboard).await;

                                let incoming_message = incoming_message(true, &mock_server, &table);
                                let event = LambdaEvent::new(incoming_message, Context::default());
                                let result = bot_lambda_handler(event).await;

                                assert_matches!(result, Ok(OutgoingMessage { output }) => {
                                    assert_eq!(output.year, YEAR);
                                    assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
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
                                assert_eq!(table.load_leaderboard().await, previous_leaderboard);
                            },
                        );
                    }
                }
            }
        }
    }

    mod using_environment {
        use std::env;

        use aoc_leaderbot_aws_lambda_impl::leaderbot::CONFIG_ENV_VAR_PREFIX;
        use aoc_leaderbot_lib::leaderbot::config::env::{
            ENV_CONFIG_AOC_SESSION_SUFFIX, ENV_CONFIG_LEADERBOARD_ID_SUFFIX, ENV_CONFIG_YEAR_SUFFIX,
        };
        use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
            CHANNEL_ENV_VAR, WEBHOOK_URL_ENV_VAR,
        };

        use super::*;

        fn configure_environment(mock_server: &MockServer) {
            let set_env_config_var = |suffix, value: &str| {
                env::set_var(format!("{CONFIG_ENV_VAR_PREFIX}{suffix}"), value)
            };

            set_env_config_var(ENV_CONFIG_YEAR_SUFFIX, &YEAR.to_string());
            set_env_config_var(ENV_CONFIG_LEADERBOARD_ID_SUFFIX, &LEADERBOARD_ID.to_string());
            set_env_config_var(ENV_CONFIG_AOC_SESSION_SUFFIX, AOC_SESSION);

            env::set_var(WEBHOOK_URL_ENV_VAR, format!("{}{WEBHOOK_PATH}", mock_server.uri()));
            env::set_var(CHANNEL_ENV_VAR, CHANNEL);
        }

        fn incoming_message(mock_server: &MockServer, table: &LocalTable) -> IncomingMessage {
            IncomingMessage {
                year: None,
                leaderboard_id: None,
                aoc_session: None,
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

            // Note: we'll only run one test using environment, otherwise we would have to
            // serialize them all, and we wouldn't really gain anything since we tested all
            // scenarios in the module above.
            #[test_log::test]
            fn updates_storage_and_sends_report() {
                let previous_leaderboard = base_leaderboard();
                let current_leaderboard = leaderboard_with_new_member();
                run_bot_test(current_leaderboard.clone(), true, |mock_server, table| async move {
                    table.save_leaderboard(&previous_leaderboard).await;
                    configure_environment(&mock_server);

                    let incoming_message = incoming_message(&mock_server, &table);
                    let event = LambdaEvent::new(incoming_message, Context::default());
                    let result = bot_lambda_handler(event).await;

                    assert_matches!(result, Ok(OutgoingMessage { output }) => {
                        assert_eq!(output.year, YEAR);
                        assert_eq!(output.leaderboard_id, LEADERBOARD_ID);
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
                    assert_eq!(table.load_leaderboard().await, current_leaderboard);
                });
            }
        }
    }
}
