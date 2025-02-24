#[cfg(all(
    any(not(ci), target_os = "linux"), // These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
    feature = "__testing"              // These tests only work if you compile with the internal `__testing` feature.
))]
mod bot_lambda_handler {}
