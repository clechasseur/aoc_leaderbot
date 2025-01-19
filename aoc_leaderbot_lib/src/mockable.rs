#![doc(hidden)]

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod helpers {
    use aoc_leaderboard::aoc::Leaderboard;

    #[cfg_attr(test, mockall::concretize)]
    pub async fn get_leaderboard(
        year: i32,
        leaderboard_id: u64,
        aoc_session: &str,
    ) -> aoc_leaderboard::Result<Leaderboard> {
        Leaderboard::get(year, leaderboard_id, aoc_session).await
    }
}
