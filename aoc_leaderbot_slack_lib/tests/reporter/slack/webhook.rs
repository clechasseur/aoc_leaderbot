mod leaderboard_sort_order {
    use aoc_leaderboard::aoc::LeaderboardMember;
    use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::LeaderboardSortOrder;
    use serde_json::json;

    fn base_member<S>(name: S, id: u64) -> LeaderboardMember
    where
        S: Into<String>,
    {
        let member_json = json!({
            "name": name.into(),
            "id": id,
        });

        serde_json::from_value(member_json).unwrap()
    }

    trait LeaderboardMemberExt {
        fn with_stars(self, stars: u32) -> Self;
        fn with_local_score(self, local_score: u64) -> Self;
        fn with_last_star_ts(self, last_star_ts: i64) -> Self;
    }

    impl LeaderboardMemberExt for LeaderboardMember {
        fn with_stars(mut self, stars: u32) -> Self {
            self.stars = stars;
            self
        }

        fn with_local_score(mut self, local_score: u64) -> Self {
            self.local_score = local_score;
            self
        }

        fn with_last_star_ts(mut self, last_star_ts: i64) -> Self {
            self.last_star_ts = last_star_ts;
            self
        }
    }

    mod cmp_members {
        use std::cmp::Ordering;

        use super::*;

        mod stars {
            use super::*;

            #[test]
            fn different_stars() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_stars(1)
                    .with_local_score(100)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_stars(42)
                    .with_local_score(84)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Stars.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Greater);
            }

            #[test]
            fn same_stars_different_local_score() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_stars(42)
                    .with_local_score(100)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_stars(42)
                    .with_local_score(84)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Stars.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Less);
            }

            #[test]
            fn same_stars_and_score_different_last_star_ts() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_stars(42)
                    .with_local_score(100)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_stars(42)
                    .with_local_score(100)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Stars.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Greater);
            }

            #[test]
            fn all_fields_equal() {
                let member = base_member("Arthur Dent", 1)
                    .with_stars(42)
                    .with_local_score(100)
                    .with_last_star_ts(1000);

                let ordering = LeaderboardSortOrder::Stars.cmp_members(&member, &member);
                assert_eq!(ordering, Ordering::Equal);
            }
        }

        mod score {
            use super::*;

            #[test]
            fn different_local_score() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_local_score(100)
                    .with_stars(1)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_local_score(84)
                    .with_stars(42)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Score.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Less);
            }

            #[test]
            fn same_local_score_different_stars() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_local_score(100)
                    .with_stars(1)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_local_score(100)
                    .with_stars(42)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Score.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Greater);
            }

            #[test]
            fn same_score_and_stars_different_last_star_ts() {
                let member_1 = base_member("Arthur Dent", 1)
                    .with_local_score(100)
                    .with_stars(42)
                    .with_last_star_ts(1000);
                let member_2 = base_member("Ford Prefect", 2)
                    .with_local_score(100)
                    .with_stars(42)
                    .with_last_star_ts(2);

                let ordering = LeaderboardSortOrder::Score.cmp_members(&member_1, &member_2);
                assert_eq!(ordering, Ordering::Greater);
            }

            #[test]
            fn all_fields_equal() {
                let member = base_member("Arthur Dent", 1)
                    .with_local_score(100)
                    .with_stars(42)
                    .with_last_star_ts(1000);

                let ordering = LeaderboardSortOrder::Score.cmp_members(&member, &member);
                assert_eq!(ordering, Ordering::Equal);
            }
        }
    }

    mod member_value_text {
        use super::*;

        #[test]
        fn stars() {
            let member = base_member("Arthur Dent", 1).with_stars(42);

            let member_text = LeaderboardSortOrder::Stars.member_value_text(&member);
            assert_eq!(member_text, "42\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}");
        }

        #[test]
        fn score() {
            let member = base_member("Arthur Dent", 1).with_local_score(100);

            let member_text = LeaderboardSortOrder::Score.member_value_text(&member);
            assert_eq!(
                member_text,
                "100\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}"
            );
        }
    }

    mod header_text {
        use super::*;

        #[test]
        fn stars() {
            let header = LeaderboardSortOrder::Stars.header_text();

            assert_eq!(header, "Stars ⭐\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}");
        }

        #[test]
        fn score() {
            let header = LeaderboardSortOrder::Score.header_text();

            assert_eq!(header, "Score #\u{2007}\u{2007}\u{2007}\u{2007}\u{2007}");
        }
    }
}

mod slack_webhook_reporter {
    use std::env;
    use std::ffi::OsStr;

    use anyhow::anyhow;
    use aoc_leaderboard::aoc::{Leaderboard, LeaderboardMember};
    use aoc_leaderboard::test_helpers::{TEST_LEADERBOARD_ID, TEST_YEAR};
    use aoc_leaderboard::wiremock::matchers::{header, method, path};
    use aoc_leaderboard::wiremock::{Mock, MockServer, ResponseTemplate};
    use aoc_leaderbot_lib::error::StorageError;
    use aoc_leaderbot_lib::leaderbot::{Changes, Reporter};
    use aoc_leaderbot_slack_lib::error::WebhookError;
    use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
        LeaderboardSortOrder, SlackWebhookReporter, SlackWebhookReporterBuilderError,
        CHANNEL_ENV_VAR, SORT_ORDER_ENV_VAR, WEBHOOK_URL_ENV_VAR,
    };
    use aoc_leaderbot_slack_lib::Error;
    use assert_matches::assert_matches;
    use reqwest::{Method, StatusCode};
    use rstest::{fixture, rstest};
    use serde_json::json;
    use serial_test::serial;
    use tracing_test::traced_test;

    const WEBHOOK_PATH: &str = "/webhook";
    const CHANNEL: &str = "#aoc_leaderbot_test";
    const USERNAME: &str = "AoC Leaderbot (test)";
    const ICON_URL: &str = "https://www.adventofcode.com/favicon.ico";

    const OWNER_NAME: &str = "Ford Prefect";
    const OWNER_ID: u64 = 1;

    const PROGRESSING_MEMBER_NAME: &str = "Zaphod Beeblebrox";
    const PROGRESSING_MEMBER_ID: u64 = 2;

    const NEW_MEMBER_ID: u64 = 3;

    #[fixture]
    async fn working_mock_server() -> MockServer {
        let mock_server = MockServer::start().await;

        Mock::given(method(Method::POST))
            .and(path(WEBHOOK_PATH))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        mock_server
    }

    fn reporter(
        mock_server: &MockServer,
        sort_order: Option<LeaderboardSortOrder>,
    ) -> SlackWebhookReporter {
        let mut builder = SlackWebhookReporter::builder();
        builder
            .webhook_url(format!("{}{}", mock_server.uri(), WEBHOOK_PATH))
            .channel(CHANNEL)
            .username(USERNAME)
            .icon_url(ICON_URL);
        if let Some(sort_order) = sort_order {
            builder.sort_order(sort_order);
        }
        builder.build().unwrap()
    }

    fn offline_reporter(mock_server: &MockServer) -> SlackWebhookReporter {
        SlackWebhookReporter::builder()
            .webhook_url(format!("{}{}", mock_server.uri(), "/invalid-path"))
            .channel(CHANNEL)
            .username(USERNAME)
            .icon_url(ICON_URL)
            .build()
            .unwrap()
    }

    #[fixture]
    fn owner() -> LeaderboardMember {
        let member_json = json!({
            "name": OWNER_NAME,
            "id": OWNER_ID,
        });

        serde_json::from_value(member_json).unwrap()
    }

    #[fixture]
    fn progressing_member() -> LeaderboardMember {
        let member_json = json!({
            "name": PROGRESSING_MEMBER_NAME,
            "id": PROGRESSING_MEMBER_ID,
        });

        serde_json::from_value(member_json).unwrap()
    }

    #[fixture]
    fn new_member(
        #[default(42)] stars: u32,
        #[default(100)] local_score: u64,
    ) -> LeaderboardMember {
        let member_json = json!({
            "id": NEW_MEMBER_ID,
            "stars": stars,
            "local_score": local_score,
        });

        serde_json::from_value(member_json).unwrap()
    }

    fn set_optional_env_var<K, V>(key: K, value: Option<V>)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }

    fn set_reporter_env_vars<W, C, S>(
        webhook_url: Option<W>,
        channel: Option<C>,
        sort_order: Option<S>,
    ) where
        W: AsRef<OsStr>,
        C: AsRef<OsStr>,
        S: AsRef<OsStr>,
    {
        set_optional_env_var(WEBHOOK_URL_ENV_VAR, webhook_url);
        set_optional_env_var(CHANNEL_ENV_VAR, channel);
        set_optional_env_var(SORT_ORDER_ENV_VAR, sort_order);
    }

    mod builder {
        use super::*;

        #[cfg(unix)]
        pub fn get_invalid_os_string() -> std::ffi::OsString {
            use std::os::unix::ffi::OsStrExt;

            // See documentation for OsString::to_string_lossy for details
            let source = [0x66, 0x6f, 0x80, 0x6f];
            std::ffi::OsString::from(std::ffi::OsStr::from_bytes(&source))
        }

        #[cfg(windows)]
        pub fn get_invalid_os_string() -> std::ffi::OsString {
            use std::os::windows::ffi::OsStringExt;

            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            std::ffi::OsString::from_wide(&source)
        }

        #[test]
        #[serial(slack_webhook_reporter_env)]
        fn with_correct_defaults() {
            set_reporter_env_vars(
                Some("https://webhook-url"),
                Some("#aoc_leaderbot_test"),
                None::<&OsStr>,
            );

            let result = SlackWebhookReporter::builder().build();
            assert!(result.is_ok());
        }

        #[test]
        #[serial(slack_webhook_reporter_env)]
        fn with_all_fields() {
            let result = SlackWebhookReporter::builder()
                .webhook_url("https://webhook-url")
                .channel("#aoc_leaderbot_test")
                .username("AoC Leaderbot (test)")
                .icon_url("https://www.adventofcode.com/favicon.ico")
                .sort_order(LeaderboardSortOrder::Score)
                .build();
            assert!(result.is_ok());
        }

        mod missing_field {
            use super::*;

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn webhook_url() {
                set_reporter_env_vars(None::<&OsStr>, Some("#aoc_leaderbot_test"), None::<&OsStr>);

                let result = SlackWebhookReporter::builder().build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message.starts_with(&format!(
                        "error reading environment variable {WEBHOOK_URL_ENV_VAR} (needed for default value of field 'webhook_url'):"
                    ))
                );
            }

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn channel() {
                set_reporter_env_vars(Some("https://webhook-url"), None::<&OsStr>, None::<&OsStr>);

                let result = SlackWebhookReporter::builder().build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message.starts_with(&format!(
                        "error reading environment variable {CHANNEL_ENV_VAR} (needed for default value of field 'channel'):"
                    ))
                );
            }
        }

        mod invalid_fields {
            use super::*;

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn invalid_sort_order_value() {
                set_reporter_env_vars(
                    None::<&OsStr>,
                    None::<&OsStr>,
                    Some("not_a_sort_order_value"),
                );

                let result = SlackWebhookReporter::builder()
                    .webhook_url("https://webhook-url")
                    .channel("#aoc_leaderbot_test")
                    .build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message == format!("invalid sort_order specified in environment variable {SORT_ORDER_ENV_VAR}: not_a_sort_order_value")
                );
            }

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn invalid_sort_order_unicode() {
                set_reporter_env_vars(
                    None::<&OsStr>,
                    None::<&OsStr>,
                    Some(get_invalid_os_string()),
                );

                let result = SlackWebhookReporter::builder()
                    .webhook_url("https://webhook-url")
                    .channel("#aoc_leaderbot_test")
                    .build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message.starts_with(&format!(
                        "invalid unicode found in environment variable {SORT_ORDER_ENV_VAR}:"
                    ))
                );
            }
        }
    }

    mod reporter {
        use super::*;

        mod report_changes {
            use super::*;

            mod valid_data {
                use super::*;

                #[rstest]
                #[case::default(None)]
                #[case::stars(Some(LeaderboardSortOrder::Stars))]
                #[case::score(Some(LeaderboardSortOrder::Score))]
                #[awt]
                #[tokio::test]
                #[serial(slack_webhook_reporter_env)]
                async fn sorted_by(
                    #[case] sort_order: Option<LeaderboardSortOrder>,
                    #[future]
                    #[from(working_mock_server)]
                    mock_server: MockServer,
                    owner: LeaderboardMember,
                    progressing_member: LeaderboardMember,
                    new_member: LeaderboardMember,
                ) {
                    set_reporter_env_vars(None::<&OsStr>, None::<&OsStr>, None::<&OsStr>);

                    let mut reporter = reporter(&mock_server, sort_order);

                    let previous_leaderboard = Leaderboard {
                        year: TEST_YEAR,
                        owner_id: owner.id,
                        day1_ts: 0,
                        members: [(owner.id, owner), (progressing_member.id, progressing_member)]
                            .into(),
                    };

                    let mut leaderboard = previous_leaderboard.clone();
                    leaderboard.members.insert(new_member.id, new_member);
                    leaderboard
                        .members
                        .get_mut(&PROGRESSING_MEMBER_ID)
                        .unwrap()
                        .stars += 1;

                    let changes = Changes {
                        new_members: [NEW_MEMBER_ID].into(),
                        members_with_new_stars: [PROGRESSING_MEMBER_ID].into(),
                    };

                    let result = reporter
                        .report_changes(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            &previous_leaderboard,
                            &leaderboard,
                            &changes,
                        )
                        .await;
                    assert!(result.is_ok());
                }
            }

            mod errors {
                use super::*;

                #[rstest]
                #[awt]
                #[tokio::test]
                #[serial(slack_webhook_reporter_env)]
                async fn not_found(
                    #[future]
                    #[from(working_mock_server)]
                    mock_server: MockServer,
                    owner: LeaderboardMember,
                    new_member: LeaderboardMember,
                ) {
                    set_reporter_env_vars(None::<&OsStr>, None::<&OsStr>, None::<&OsStr>);

                    let mut reporter = offline_reporter(&mock_server);

                    let previous_leaderboard = Leaderboard {
                        year: TEST_YEAR,
                        owner_id: owner.id,
                        day1_ts: 0,
                        members: [(owner.id, owner)].into(),
                    };

                    let mut leaderboard = previous_leaderboard.clone();
                    leaderboard.members.insert(new_member.id, new_member);
                    leaderboard.members.get_mut(&OWNER_ID).unwrap().stars += 1;

                    let changes = Changes {
                        new_members: [NEW_MEMBER_ID].into(),
                        members_with_new_stars: [OWNER_ID].into(),
                    };

                    let result = reporter
                        .report_changes(
                            TEST_YEAR,
                            TEST_LEADERBOARD_ID,
                            &previous_leaderboard,
                            &leaderboard,
                            &changes,
                        )
                        .await;
                    assert_matches!(
                        result,
                        Err(Error::Webhook(WebhookError::ReportChanges {
                            year,
                            leaderboard_id,
                            webhook_url,
                            channel,
                            source,
                        })) => {
                            assert_eq!(year, TEST_YEAR);
                            assert_eq!(leaderboard_id, TEST_LEADERBOARD_ID);
                            assert_eq!(webhook_url, format!("{}/invalid-path", mock_server.uri()));
                            assert_eq!(channel, CHANNEL);
                            assert!(source.is_status());
                            assert_matches!(source.status(), Some(StatusCode::NOT_FOUND));
                        }
                    );
                }
            }
        }

        mod report_error {
            use super::*;

            #[rstest]
            #[awt]
            #[tokio::test]
            #[traced_test]
            #[serial(slack_webhook_reporter_env)]
            async fn working(
                #[future]
                #[from(working_mock_server)]
                mock_server: MockServer,
            ) {
                set_reporter_env_vars(None::<&OsStr>, None::<&OsStr>, None::<&OsStr>);

                let mut reporter = reporter(&mock_server, None);

                let error = aoc_leaderbot_lib::Error::Storage(StorageError::LoadPrevious(anyhow!(
                    "something is wrong"
                )));
                reporter
                    .report_error(TEST_YEAR, TEST_LEADERBOARD_ID, &error)
                    .await;

                assert!(logs_contain(&format!(
                    "error for leaderboard {TEST_LEADERBOARD_ID} and year {TEST_YEAR}: {error}"
                )));
                assert!(!logs_contain(&format!("error trying to report previous error to Slack webhook for leaderboard {TEST_LEADERBOARD_ID} and year {TEST_YEAR}")));
            }

            #[rstest]
            #[awt]
            #[tokio::test]
            #[traced_test]
            #[serial(slack_webhook_reporter_env)]
            async fn offline(
                #[future]
                #[from(working_mock_server)]
                mock_server: MockServer,
            ) {
                set_reporter_env_vars(None::<&OsStr>, None::<&OsStr>, None::<&OsStr>);

                let mut reporter = offline_reporter(&mock_server);

                let error = aoc_leaderbot_lib::Error::Storage(StorageError::LoadPrevious(anyhow!(
                    "something is wrong"
                )));
                reporter
                    .report_error(TEST_YEAR, TEST_LEADERBOARD_ID, &error)
                    .await;

                assert!(logs_contain(&format!(
                    "error for leaderboard {TEST_LEADERBOARD_ID} and year {TEST_YEAR}: {error}"
                )));
                assert!(logs_contain(&format!("error trying to report previous error to Slack webhook for leaderboard {TEST_LEADERBOARD_ID} and year {TEST_YEAR}")));
            }
        }
    }
}
