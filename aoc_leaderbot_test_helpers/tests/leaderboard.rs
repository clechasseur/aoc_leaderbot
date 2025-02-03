mod get_sample_leaderboard {
    use aoc_leaderbot_test_helpers::get_sample_leaderboard;

    #[test]
    fn can_load() {
        let leaderboard = get_sample_leaderboard();
        assert!(!leaderboard.members.is_empty());
    }
}
