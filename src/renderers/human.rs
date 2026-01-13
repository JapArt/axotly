use crate::domain::{TestCase, TestResult, Renderer};
use crate::renderers::response::ResponseRenderer;
use std::time::Duration;
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub struct HumanRenderer;

impl HumanRenderer {
    pub fn new() -> Self {
        Self
    }

    fn fmt_duration(d: &Duration) -> String {
        if d.as_millis() < 1000 {
            format!("{}ms", d.as_millis())
        } else {
            format!("{:.2}s", d.as_secs_f64())
        }
    }
}

impl Renderer for HumanRenderer {
    fn test(&self, test: &TestCase, file: Option<&PathBuf>) {
        if let Some(file) = file {
            println!("\n{}", file.to_string_lossy().dimmed());
        }

        let name = test.name.as_deref().unwrap_or("<unnamed>");

        match &test.result {
            Some(TestResult::Passed { duration }) => {
                println!(
                    "{} {} {}",
                    "✓".green().bold(),
                    name.bold(),
                    format!("({})", Self::fmt_duration(duration))
                        .dimmed()
                );
            }

            Some(TestResult::Failed { duration, errors: _ }) => {
                println!(
                    "{} {} {}",
                    "✗".red().bold(),
                    name.bold(),
                    format!("({})", Self::fmt_duration(duration))
                        .dimmed()
                );
            }

            None => {
                println!(
                    "{} {}",
                    "?".yellow(),
                    name.yellow()
                );
            }
        }
    }

    fn summary(&self, tests: &[TestCase], total_duration: &Duration) {
        let mut passed = 0;
        let mut failed = 0;
        let mut total = Duration::ZERO;

        for test in tests {
            if let Some(result) = &test.result {
                match result {
                    TestResult::Passed { duration } => {
                        passed += 1;
                        total += *duration;
                    }
                    TestResult::Failed { duration, .. } => {
                        failed += 1;
                        total += *duration;
                    }
                }
            }
        }

        if failed > 0 {
            println!("\n{}", "Failures".red().bold());

            let mut idx = 1;
            for test in tests {
                if let Some(TestResult::Failed { errors, duration }) = &test.result {
                    let name = test.name.as_deref().unwrap_or("<unnamed>");

                    println!(
                        "\n{} {} {}",
                        format!("{})", idx).red().bold(),
                        name.bold(),
                        format!("({})", Self::fmt_duration(duration))
                            .dimmed()
                    );

                    for error in errors {
                        println!(
                            "  {} {}",
                            "-".red(),
                            error.message
                        );
                    }

                    if let Some(response) = &test.response {
                       ResponseRenderer::print_response(response);
                    }

                    idx += 1;
                }
            }
        }
        
        println!("\n{}", "────────────────────────────────────".dimmed());
        println!("{}", "Results".bold());
        println!(
            "{} {}",
            "✓ Passed:".green(),
            passed.to_string().green().bold()
        );
        println!(
            "{} {}",
            "✗ Failed:".red(),
            failed.to_string().red().bold()
        );
        println!(
            "{} {}",
            "⏱ Total requests duration:".magenta(),
            Self::fmt_duration(&total).magenta().bold()
        );
        println!("{}", "────────────────────────────────────".dimmed());
        
        println!(
            "Test suite completed in: {}",
            format!("{:.2?}", total_duration).bold()
        );
    }
}

