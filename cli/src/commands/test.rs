//! `forge test` — Run the test suite.

use anyhow::Result;
use clap::Args;
use forge_runtime::isolate::v8_runtime::ForgeRuntime;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Args)]
pub struct TestArgs {
    /// Watch mode: re-run tests on file changes
    #[arg(long, short)]
    pub watch: bool,
    /// Run only tests matching this pattern
    pub filter: Option<String>,
}

fn find_tests(dir: &Path, filter: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut tests = Vec::new();
    let entries = std::fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name != "node_modules" && name != "dist" && name != "target" {
                if let Ok(mut sub_tests) = find_tests(&path, filter) {
                    tests.append(&mut sub_tests);
                }
            }
        } else if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "fx" {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.ends_with(".test.ts") || name.ends_with(".test.fx") {
                    if let Some(f) = filter {
                        if !name.contains(f) && !path.to_string_lossy().contains(f) {
                            continue;
                        }
                    }
                    tests.push(path);
                }
            }
        }
    }
    Ok(tests)
}

async fn run_tests(filter: Option<&str>) -> Result<()> {
    crate::output::info("Discovering tests...");
    let mut tests = find_tests(Path::new("."), filter).unwrap_or_default();
    tests.sort();

    if tests.is_empty() {
        crate::output::warn("No tests found matching the criteria.");
        return Ok(());
    }

    crate::output::info(&format!("Found {} test file(s)", tests.len()));
    let mut passed = 0;
    let mut failed = 0;

    // Create a new ForgeRuntime to execute tests
    let mut runtime = ForgeRuntime::new()?;

    for test in &tests {
        crate::output::info(&format!("Running {}", test.display()));

        let content = tokio::fs::read(test).await?;
        // Execute the test using the runtime on the actual content
        // Note: Currently ForgeRuntime::execute_module is a stub, but we prepare for real execution.
        let execution_result = runtime.execute_module(&content).await;

        match execution_result {
            Ok(_) => {
                crate::output::success(&format!("PASS {}", test.display()));
                passed += 1;
            }
            Err(e) => {
                crate::output::error(&format!("FAIL {}: {}", test.display(), e));
                failed += 1;
            }
        }
    }

    if failed == 0 {
        crate::output::success(&format!("\nTest Summary: {} passed, 0 failed", passed));
        Ok(())
    } else {
        crate::output::error(&format!(
            "\nTest Summary: {} passed, {} failed",
            passed, failed
        ));
        anyhow::bail!("{} test(s) failed", failed);
    }
}

pub async fn run(args: TestArgs) -> Result<()> {
    let filter = args.filter.as_deref();

    if let Err(e) = run_tests(filter).await {
        if !args.watch {
            return Err(e);
        }
    }

    if args.watch {
        crate::output::info("Watching for file changes...");
        // Use an unbounded channel to prevent blocking the watcher loop
        let (tx, mut rx) = mpsc::unbounded_channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| match res {
                Ok(event) => {
                    let _ = tx.send(event);
                }
                Err(err) => {
                    eprintln!("watch error: {err}");
                }
            },
            Config::default(),
        )?;

        watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

        // Simple debounce loop
        while let Some(event) = rx.recv().await {
            let mut should_rerun = false;

            let check_event = |event: notify::Event| -> bool {
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    for path in event.paths {
                        // Check exclusions
                        let mut excluded = false;
                        for comp in path.components() {
                            let name = comp.as_os_str().to_string_lossy();
                            if name == "node_modules" || name == "dist" || name == "target" {
                                excluded = true;
                                break;
                            }
                        }

                        if excluded {
                            continue;
                        }

                        if let Some(ext) = path.extension() {
                            if ext == "ts" || ext == "fx" {
                                return true;
                            }
                        }
                    }
                }
                false
            };

            if check_event(event) {
                should_rerun = true;
            }

            // Wait briefly for subsequent events to accumulate
            sleep(Duration::from_millis(100)).await;

            // Drain any additional events that occurred during the sleep
            while let Ok(event) = rx.try_recv() {
                if !should_rerun && check_event(event) {
                    should_rerun = true;
                }
            }

            if should_rerun {
                crate::output::info("\nFile change detected. Re-running tests...");
                if let Err(err) = run_tests(filter).await {
                    eprintln!("Error re-running tests: {err}");
                }
            }
        }
    }

    Ok(())
}
