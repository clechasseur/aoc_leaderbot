mod webhook_message {
    mod builder {
        use aoc_leaderbot_slack_lib::error::WebhookError;
        use aoc_leaderbot_slack_lib::slack::webhook::{WebhookMessage, WebhookMessageBuilderError};
        use aoc_leaderbot_slack_lib::Error;
        use assert_matches::assert_matches;

        #[test]
        fn without_message_text() {
            let result = WebhookMessage::builder().build();

            assert_matches!(
                result,
                Err(Error::Webhook(WebhookError::MessageBuilder(
                    WebhookMessageBuilderError::UninitializedField("text")
                )))
            );
        }

        #[test]
        fn with_message_text() {
            let result = WebhookMessage::builder()
                .text("Hello from aoc_leaderbot!")
                .build();

            let expected = WebhookMessage {
                channel: None,
                username: None,
                icon_url: None,
                text: "Hello from aoc_leaderbot!".into(),
            };
            assert_matches!(result, Ok(actual) if actual == expected);
        }

        #[test]
        fn with_all_fields() {
            let result = WebhookMessage::builder()
                .channel("#aoc_leaderbot_test")
                .username("AoC Leaderbot (test)")
                .icon_url("https://www.adventofcode.com/favicon.ico")
                .text("Hello from aoc_leaderbot!")
                .build();

            let expected = WebhookMessage {
                channel: Some("#aoc_leaderbot_test".into()),
                username: Some("AoC Leaderbot (test)".into()),
                icon_url: Some("https://www.adventofcode.com/favicon.ico".into()),
                text: "Hello from aoc_leaderbot!".into(),
            };
            assert_matches!(result, Ok(actual) if actual == expected);
        }
    }
}
