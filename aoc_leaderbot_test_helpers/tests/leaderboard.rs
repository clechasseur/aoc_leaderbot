mod get_sample_leaderboard {
    use aoc_leaderbot_test_helpers::get_sample_leaderboard;

    #[test_log::test]
    fn can_load() {
        let leaderboard = get_sample_leaderboard();
        assert!(!leaderboard.members.is_empty());
    }
}

mod get_mock_server_with_sample_leaderboard {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_test_helpers::{
        get_mock_server_with_sample_leaderboard, get_sample_leaderboard, AOC_SESSION,
        LEADERBOARD_ID, YEAR,
    };

    #[test_log::test(tokio::test)]
    async fn can_load_leaderboard() {
        let mock_server = get_mock_server_with_sample_leaderboard().await;

        let expected = get_sample_leaderboard();
        let actual = Leaderboard::get_from(
            Leaderboard::http_client().unwrap(),
            mock_server.uri(),
            YEAR,
            LEADERBOARD_ID,
            AOC_SESSION,
        )
        .await
        .unwrap();
        assert_eq!(expected, actual);
    }
}

mod get_mock_server_with_inaccessible_leaderboard {
    use aoc_leaderboard::aoc::Leaderboard;
    use aoc_leaderbot_test_helpers::{
        get_mock_server_with_inaccessible_leaderboard, AOC_SESSION, LEADERBOARD_ID, YEAR,
    };
    use assert_matches::assert_matches;

    #[test_log::test(tokio::test)]
    async fn cannot_load_leaderboard() {
        let mock_server = get_mock_server_with_inaccessible_leaderboard().await;

        let result = Leaderboard::get_from(
            Leaderboard::http_client().unwrap(),
            mock_server.uri(),
            YEAR,
            LEADERBOARD_ID,
            AOC_SESSION,
        )
        .await;
        assert_matches!(result, Err(aoc_leaderboard::Error::NoAccess));
    }
}
