mod get_sample_leaderboard {
    use aoc_leaderbot_test_helpers::get_sample_leaderboard;

    #[test]
    fn can_load() {
        let leaderboard = get_sample_leaderboard();
        assert!(!leaderboard.members.is_empty());
    }
}

mod get_mock_server_with_leaderboard {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_test_helpers::{
        get_mock_server_with_leaderboard, get_sample_leaderboard, AOC_SESSION, LEADERBOARD_ID, YEAR,
    };
    use reqwest::header;

    #[tokio::test]
    async fn can_load_leaderboard() {
        let mock_server = get_mock_server_with_leaderboard().await;

        let expected = get_sample_leaderboard();
        let actual: Leaderboard = reqwest::Client::new()
            .get(format!(
                "{}/{YEAR}/leaderboard/private/view/{LEADERBOARD_ID}.json",
                mock_server.uri()
            ))
            .header(header::COOKIE, format!("session={AOC_SESSION}"))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
}
