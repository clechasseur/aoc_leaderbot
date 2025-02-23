// These tests require Docker, which only seems to work reliably on Linux in GitHub workflows.
#[cfg(any(not(ci), target_os = "linux"))]
mod bot_lambda_handler {}
