//! `forge test` — Run the test suite.

use anyhow::Result;
use clap::Args;
use forge_runtime::isolate::v8_runtime::ForgeRuntime;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

#[derive(Debug, Args)]
pub struct TestArgs {
    /// Watch mode: re-run tests on file changes
    #[arg(long, short)]
    pub watch: bool,
    /// Run only tests matching this pattern
    pub filter: Option<String>,
}

fn find_tests<'a>(
    dir: &'a Path,
    filter: Option<&'a str>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + Send + 'a>> {
    Box::pin(async move {
        let mut tests = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "node_modules" && name != "dist" && name != "target" {
                    tests.extend(find_tests(&path, filter).await?);
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
    })
}

async fn run_tests(filter: Option<&str>) -> Result<()> {
    crate::output::info("Discovering tests...");
    let mut tests = find_tests(Path::new("."), filter).await?;
    tests.sort();

    if tests.is_empty() {
        crate::output::warn("No tests found matching the criteria.");
        return Ok(());
    }

    crate::output::info(&format!("Found {} test file(s)", tests.len()));
    let mut passed = 0;
    let mut failed = 0;

    let mut runtime = ForgeRuntime::new()?;

    for test in &tests {
        crate::output::info(&format!("Running {}", test.display()));

        let content = tokio::fs::read(test).await?;
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
    } else {
        crate::output::error(&format!(
            "\nTest Summary: {} passed, {} failed",
            passed, failed
        ));
        anyhow::bail!("{} test(s) failed", failed);
    }

    Ok(())
}

fn is_relevant_event(event: &notify::Event) -> bool {
    const EXCLUDED: &[&str] = &["node_modules", "dist", "target"];
    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
        for path in &event.paths {
            if path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                EXCLUDED.contains(&s.as_ref())
            }) {
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
}

pub async fn run(args: TestArgs) -> Result<()> {
    let filter = args.filter.as_deref();
    run_tests(filter).await?;

    if args.watch {
        watch_loop(filter).await?;
    }

    Ok(())
}

async fn watch_loop(filter: Option<&str>) -> Result<()> {
    crate::output::info("Watching for file changes...");
    let notify = Arc::new(Notify::new());
    let notify_clone = Arc::clone(&notify);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                if is_relevant_event(&event) {
                    notify_clone.notify_one();
                }
            }
            Err(err) => {
                eprintln!("watch error: {err}");
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    loop {
        notify.notified().await;
        sleep(Duration::from_millis(100)).await;
        crate::output::info("\nFile change detected. Re-running tests...");
        if let Err(err) = run_tests(filter).await {
            eprintln!("Error re-running tests: {err}");
        }
    }
}
