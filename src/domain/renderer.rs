use crate::domain::TestCase;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Duration;


pub trait Renderer {
    fn start(&self, total: usize) {
        println!("\n{}", "Axotly — API tests".bold());
        println!("Running {} tests...", total);
    }

    fn test(&self, test: &TestCase, file: Option<&PathBuf>);

    fn summary(&self, tests: &[TestCase], total_duration: &Duration );
}
