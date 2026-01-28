//! Command-line entry point and execution modes.
//!
//! This binary provides a CLI for executing HTTP-based tests defined in `.ax`
//! files or for running a single ad-hoc HTTP request directly from the command
//! line.
//!
//! The application supports two primary modes of operation:
//!
//! ## 1. File / folder execution mode
//!
//! When a file or directory path is provided via CLI arguments:
//!
//! - `.ax` test files are discovered and parsed
//! - Tests are executed with bounded concurrency
//! - Results are rendered incrementally
//! - A final summary is produced
//!
//! This mode is orchestrated by the [`Runner`] and is intended for batch test
//! execution.
//!
//! ## Rendering
//!
//! Output formatting is delegated to a pluggable [`Renderer`] implementation,
//! selected at runtime via CLI options. This allows the same execution pipeline
//! to support multiple output styles (human-readable, diff-based, etc.).
//!
//! ## 2. Single request mode
//!
//! When no file path is provided:
//!
//! - A single HTTP request is constructed from CLI arguments
//! - Optional request bodies (`--body` or `--json`) are supported
//! - The request is executed immediately
//!
//! This mode is useful for quick inspection, debugging, or exploratory calls.


mod cli;
mod domain;
mod parser;
mod executor;
mod renderers;
mod runner;

use anyhow::Result;
use cli::{Cli, RendererKind};
use clap::Parser;
use domain::{
    http_request::{Body, HttpRequest, HttpResponse},
    Renderer,
};
use renderers::human::HumanRenderer;
use renderers::diff::DiffRenderer;
use renderers::response::ResponseRenderer;
use runner::Runner;
use url::Url;

async fn handle_file_request(
    path: String,
    max_concurrency: usize,
    renderer: &dyn Renderer,
    show_response: bool,
) -> Result<()> {
    Runner::run_path(path, max_concurrency, renderer, show_response).await?;
    Ok(())
}

async fn handle_single_request(args: &Cli) -> Result<()> {
    let url = args.url.clone().unwrap_or_else(|| {
        "http://httpbin.org/get".to_string()
    });

    if args.json.is_some() && args.body.is_some() {
        anyhow::bail!("Cannot use both --body and --json options together.");
    }

    let mut body_content: Option<Body> = None;

    if args.json.is_some() {
        let json_str = args.json.clone().unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Invalid JSON body: {}", e))?;
        body_content = Some(Body::Json(json_value));
    }

    if args.body.is_some() {
        body_content = Some(Body::Text(args.body.clone().unwrap()));
    }

    let request = HttpRequest::new(args.method.clone(), Url::parse(&url)?)
        .body(body_content);

    let response: HttpResponse = request.send().await?;
    ResponseRenderer::print_response(&response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    
    let renderer: Box<dyn Renderer> = match args.renderer {
        RendererKind::Human => Box::new(HumanRenderer::new()),
        RendererKind::Diff => Box::new(DiffRenderer::new()),
    };

    if let Some(path) = args.file {
        handle_file_request(path, args.concurrently, renderer.as_ref(), args.show_response).await?;
    } else {
        // Single request mode
        handle_single_request(&args).await?;
    }

    Ok(())
}

