//! `forge test` — Run the test suite.

use anyhow::Result;
use clap::Args;
use forge_runtime::test::{run_tests, TestOptions};

#[derive(Debug, Args)]
pub struct TestArgs {
    /// Watch mode: re-run tests on file changes
    #[arg(long, short)]
    pub watch: bool,
    /// Run only tests matching this pattern
    pub filter: Option<String>,
}

pub async fn run(args: TestArgs) -> Result<()> {
    let options = TestOptions {
        watch: args.watch,
        filter: args.filter,
    };

    // Delegate to the forge-runtime test runner implementation.
    run_tests(options).await?;

    Ok(())
}
