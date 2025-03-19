# aoc_leaderboard

[![CI](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/aoc_leaderbot/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/aoc_leaderbot) [![Security audit](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/aoc_leaderbot/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/aoc_leaderboard.svg)](https://crates.io/crates/aoc_leaderboard) [![downloads](https://img.shields.io/crates/d/aoc_leaderboard.svg)](https://crates.io/crates/aoc_leaderboard) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aoc_leaderboard) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](../CODE_OF_CONDUCT.md)

Strongly-typed wrapper for an [Advent of Code](https://adventofcode.com/) leaderboard and a convenient way to fetch its data.

## Installing

Add `aoc_leaderboard` to your dependencies:

```toml
[dependencies]
# Enable http feature to be able to fetch leaderboard data
aoc_leaderboard = { version = "0.3.0", features = ["http"] }
```

or by running:

```bash
cargo add aoc_leaderboard --features http
```

## Example

```rust
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
```

The above example is available [here](./examples/http.rs).
For complete API usage, see [the docs](https://docs.rs/aoc_leaderboard).

## Minimum Rust version

`aoc_leaderboard` currently builds on Rust 1.74.1 or newer.

## Contributing / Local development

For information about contributing to this project, see [CONTRIBUTING](../CONTRIBUTING.md).
For information regarding local development, see [DEVELOPMENT](../DEVELOPMENT.md).
