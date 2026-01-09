use std::path::Path;
use walkdir::WalkDir;
use anyhow::{Result, Context};

use crate::domain::test_case::TestCase;
use crate::domain::renderer::Renderer;
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
