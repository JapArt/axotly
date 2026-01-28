//! Test discovery and execution orchestration.
//!
//! This module defines the [`Runner`], the high-level coordinator responsible
//! for discovering `.ax` test files, parsing them into [`TestCase`]s, executing
//! them with controlled concurrency, and rendering results.
//!
//! The runner acts as the glue between:
//!
//! - **Input**: a single `.ax` file or a directory containing multiple files
//! - **Parsing**: converting `.ax` files into executable [`TestCase`]s
//! - **Execution**: delegating concurrent execution to the [`Executor`]
//! - **Rendering**: streaming test results and final summaries via a [`Renderer`]
//!
//! ## Responsibilities
//!
//! - Walk directories recursively to discover `.ax` files
//! - Load and parse tests per file
//! - Execute tests with bounded concurrency
//! - Render results incrementally as tests complete
//! - Produce a final summary across all executed tests
//!
//! The runner itself is intentionally stateless and does not retain global
//! execution context. All state is scoped to a single invocation.
//!
//! ## Execution flow
//!
//! 1. Determine whether the provided path is a file or directory.
//! 2. Discover all `.ax` files (recursively for directories).
//! 3. Parse each file into a list of [`TestCase`]s.
//! 4. Initialize the renderer and start timing.
//! 5. Execute tests file-by-file using the [`Executor`].
//! 6. Render each test result as it completes.
//! 7. Emit a final summary with aggregated results and duration.
//!
//! ## Output behavior
//!
//! - Test results are rendered immediately after execution.
//! - File boundaries are printed to provide visual grouping.
//! - Optional HTTP responses can be printed when `show_response` is enabled.
//!
//! Errors while reading or parsing files are surfaced immediately and stop
//! execution.

use std::path::Path;
use walkdir::WalkDir;
use anyhow::{Result, Context};

use crate::domain::test_case::TestCase;
use crate::domain::renderer::Renderer;
use crate::renderers::response::ResponseRenderer;
use crate::executor::Executor;
use crate::parser::AxParser;
use owo_colors::OwoColorize;

pub struct Runner;

impl Runner {
    /// Run tests from a single file or folder and produce a single summary
    pub async fn run_path<P: AsRef<Path>>(
        path: P,
        max_concurrency: usize,
        renderer: &dyn Renderer,
        show_response: bool,
    ) -> Result<()> {
        let path = path.as_ref();

        // Gather all tests with their file paths
        let mut all_tests = Vec::new();
        let mut all_results = Vec::new();

        if path.is_file() {
            let tests = Self::load_tests_from_file(path)?;
            all_tests.push((path.to_path_buf(), tests));
        } else if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.extension().map(|ext| ext == "ax").unwrap_or(false) {
                    let tests = Self::load_tests_from_file(entry_path)?;
                    all_tests.push((entry_path.to_path_buf(), tests));
                }
            }
        } else {
            anyhow::bail!("{} is neither a file nor a folder", path.display());
        }

        if all_tests.is_empty() {
            println!("No tests found in {}", path.display());
            return Ok(());
        }

        // Count total tests
        let total_tests: usize = all_tests.iter().map(|(_, tests)| tests.len()).sum();
        renderer.start(total_tests);
        let start_time = std::time::Instant::now();

        // Run tests per file and render immediately
        for (file_path, tests) in all_tests {
            println!("\n{}", file_path.display().dimmed());
            let results = Executor::run_tests(tests, max_concurrency).await;
            for test in &results {
                renderer.test(test, None);
                if show_response {
                    if let Some(resp) = &test.response {
                        ResponseRenderer::print_response(resp);
                    }
                }
            }
            all_results.extend(results);
        }

        let duration = start_time.elapsed();
        renderer.summary(&all_results, &duration);

        Ok(())
    }

    /// Load tests from a single .ax file
    fn load_tests_from_file(path: &Path) -> Result<Vec<TestCase>> {
        let input = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file {}", path.display()))?;
        let tests = AxParser::parse_file(&input)?;
        Ok(tests)
    }
}
