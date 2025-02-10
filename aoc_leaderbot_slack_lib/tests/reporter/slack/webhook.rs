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
    mod builder {
        mod missing_field {
            use std::env;

            use aoc_leaderbot_slack_lib::error::WebhookError;
            use aoc_leaderbot_slack_lib::leaderbot::reporter::slack::webhook::{
                SlackWebhookReporter, SlackWebhookReporterBuilderError, CHANNEL_ENV_VAR,
                WEBHOOK_URL_ENV_VAR,
            };
            use aoc_leaderbot_slack_lib::Error;
            use assert_matches::assert_matches;
            use serial_test::serial;

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn webhook_url() {
                env::remove_var(WEBHOOK_URL_ENV_VAR);

                let result = SlackWebhookReporter::builder().build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message.starts_with(
                        &format!("error reading environment variable {} (needed for default value of field webhook_url):",
                        WEBHOOK_URL_ENV_VAR)
                    )
                );
            }

            #[test]
            #[serial(slack_webhook_reporter_env)]
            fn channel() {
                env::set_var(WEBHOOK_URL_ENV_VAR, "https://webhook-url");
                env::remove_var(CHANNEL_ENV_VAR);

                let result = SlackWebhookReporter::builder().build();
                assert_matches!(
                    result,
                    Err(Error::Webhook(WebhookError::ReporterBuilder(
                        SlackWebhookReporterBuilderError::ValidationError(error_message)
                    ))) if error_message.starts_with(
                        &format!("error reading environment variable {} (needed for default value of field channel):",
                        CHANNEL_ENV_VAR)
                    )
                );
            }
        }
    }
}
