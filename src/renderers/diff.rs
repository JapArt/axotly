use crate::domain::{AssertionFailure, TestCase, TestResult, Renderer};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Duration;

pub struct DiffRenderer;

impl DiffRenderer {
    pub fn new() -> Self {
        Self
    }

    fn render_failure(&self, index: usize, failure: &AssertionFailure) {
        println!(
            "  {} {}",
            index.to_string().dimmed(),
            failure.path.bold()
        );

        let expected = failure
            .expected
            .as_deref()
            .unwrap_or("<unknown>");

        let actual = failure
            .actual
            .as_deref()
            .unwrap_or("<missing>");

        println!(
            "    {} {}",
            "- expected:".red(),
            expected.red()
        );

        println!(
            "    {} {}\n",
            "+ actual:  ".green(),
            actual.green()
        );
    }
}

impl Renderer for DiffRenderer {
    fn test(&self, test: &TestCase, _file: Option<&PathBuf>) {
        let name = test.name.as_deref().unwrap_or("<unnamed>");

        match &test.result {
            Some(TestResult::Passed { duration }) => {
                println!(
                    "{} {} ({:?})",
                    "✔".green(),
                    name.bold(),
                    duration
                );
            }

            Some(TestResult::Failed { duration, errors }) => {
                println!(
                    "{} {} ({:?})",
                    "✖".red(),
                    name.bold(),
                    duration
                );

                for (i, failure) in errors.iter().enumerate() {
                    self.render_failure(i + 1, failure);
                }
            }

            None => {}
        }
    }

    fn summary(&self, tests: &[TestCase], total_duration: &Duration) {
        let total = tests.len();
        let passed = tests
            .iter()
            .filter(|t| matches!(t.result, Some(TestResult::Passed { .. })))
            .count();
        let failed = total - passed;

        println!("{}", "─".repeat(40).dimmed());

        if failed == 0 {
            println!(
                "{} {} tests passed",
                "✔".green(),
                total.to_string().bold()
            );
        } else {
            println!(
                "{} {} passed, {} failed",
                "✖".red(),
                passed.to_string().bold(),
                failed.to_string().bold()
            );
        }
        println!(
            "Completed in: {}",
            format!("{:.2?}", total_duration).bold()
        );
    }
}
