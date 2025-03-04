//! Main executable containing the [AWS Lambda] function for [`aoc_leaderbot`].
//!
//! [AWS Lambda]: https://aws.amazon.com/lambda/
//! [`aoc_leaderbot`]: https://github.com/clechasseur/aoc_leaderbot

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use aoc_leaderbot_aws_lambda_impl::leaderbot::bot_lambda_handler;
use dotenvy::dotenv;
use lambda_runtime::{run, service_fn, tracing, Error};

#[tokio::main]
#[cfg_attr(coverage_nightly, coverage(off))]
async fn main() -> Result<(), Error> {
    let _ = dotenv();

    tracing::init_default_subscriber();

    run(service_fn(bot_lambda_handler)).await
}
