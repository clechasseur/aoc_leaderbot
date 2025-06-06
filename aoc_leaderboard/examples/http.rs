use std::env;

use aoc_leaderboard::aoc::Leaderboard;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Maybe your AoC session token lives in a `.env` file?
    let _ = dotenv();

    // Fetch AoC session token and leaderboard ID from the environment.
    let aoc_session = env::var("AOC_SESSION")?;
    let leaderboard_id = env::var("AOC_LEADERBOARD_ID")?.parse()?;

    // Load the leaderboard from the AoC website.
    // Careful not to call this more than once every **15 minutes**.
    let year = 2024;
    let leaderboard = Leaderboard::get(year, leaderboard_id, aoc_session).await?;

    // Do something useful.
    println!("Leaderboard for year {year} has {} members.", leaderboard.members.len());

    Ok(())
}
