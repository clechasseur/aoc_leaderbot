#[cfg(feature = "http")]
mod real_endpoint {
    use std::env;

    use aoc_leaderboard::aoc::{Leaderboard, LeaderboardCredentials};
    use assert_matches::assert_matches;

    fn leaderboard_id() -> Option<u64> {
        env::var("AOC_LEADERBOARD_ID")
            .ok()
            .and_then(|id| id.parse::<u64>().ok())
    }

    fn year() -> Option<i32> {
        env::var("AOC_LEADERBOARD_YEAR")
            .ok()
            .and_then(|id| id.parse::<i32>().ok())
    }

    fn credentials() -> Option<LeaderboardCredentials> {
        env::var("AOC_VIEW_KEY")
            .map(LeaderboardCredentials::ViewKey)
            .or_else(|_| env::var("AOC_SESSION").map(LeaderboardCredentials::SessionCookie))
            .ok()
    }

    #[test_log::test(tokio::test)]
    async fn test_with_real_endpoint() {
        // This test is not executed by default because we could get banned from AoC
        // (we're not supposed to ping a leaderboard's API URL more that once every 15 minutes).
        let _ = dotenvy::dotenv();

        if let (Some(leaderboard_id), Some(year), Some(credentials)) =
            (leaderboard_id(), year(), credentials())
        {
            println!("test_with_real_endpoint executed");
            let leaderboard = Leaderboard::get(year, leaderboard_id, &credentials).await;
            assert_matches!(leaderboard, Ok(leaderboard) => {
                let clechasseur = leaderboard.members.values().find(|m| m.name == Some("clechasseur".into()));
                assert_matches!(clechasseur, Some(clechasseur) => {
                    assert_eq!(clechasseur.id, leaderboard_id);
                });
            });
        }
    }
}
