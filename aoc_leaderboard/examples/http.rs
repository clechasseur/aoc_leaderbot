use std::env;

use aoc_leaderboard::aoc::{Leaderboard, LeaderboardCredentials};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Maybe your config lives in a `.env` file?
    let _ = dotenv();

    // Fetch leaderboard ID and AoC credentials from the environment.
    let leaderboard_id = env::var("AOC_LEADERBOARD_ID")?.parse()?;
    let credentials = aoc_credentials()?;

    // Load the leaderboard from the AoC website.
    // Careful not to call this more than once every **15 minutes**.
    let year = 2024;
    let leaderboard = Leaderboard::get(year, leaderboard_id, &credentials).await?;

    // Do something useful.
    println!("Leaderboard for year {year} has {} members.", leaderboard.members.len());

    Ok(())
}

fn aoc_credentials() -> anyhow::Result<LeaderboardCredentials> {
    Ok(env::var("AOC_VIEW_KEY")
        .map(LeaderboardCredentials::ViewKey)
        .or_else(|_| env::var("AOC_SESSION").map(LeaderboardCredentials::SessionCookie))?)
}
