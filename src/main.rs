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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let renderer: Box<dyn Renderer> = match args.renderer {
        RendererKind::Human => Box::new(HumanRenderer::new()),
        RendererKind::Diff => Box::new(DiffRenderer::new()),
    };

    if let Some(path) = args.file {
        let max_concurrency = args.concurrently;
        Runner::run_path(path, max_concurrency, renderer.as_ref()).await?;
        return Ok(());
    }

    // Single request mode
    let request = HttpRequest::new(args.method, Url::parse(&args.url)?)
        .body(args.body.map(Body::Text));
    let response: HttpResponse = request.send().await?;
    ResponseRenderer::print_response(&response);

    Ok(())
}

