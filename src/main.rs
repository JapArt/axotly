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
    let url = args.url.unwrap_or_else(|| {
        "http://httpbin.org/get".to_string()
    });

    if args.json.is_some() && args.body.is_some() {
        anyhow::bail!("Cannot use both --body and --json options together.");
    }

    let mut body_content: Option<Body> = None;

    if args.json.is_some() {
        let json_str = args.json.unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Invalid JSON body: {}", e))?;
        body_content = Some(Body::Json(json_value));
    }

    if args.body.is_some() {
        body_content = Some(Body::Text(args.body.clone().unwrap()));
    }

    let request = HttpRequest::new(args.method, Url::parse(&url)?)
        .body(body_content);

    let response: HttpResponse = request.send().await?;
    ResponseRenderer::print_response(&response);


    Ok(())
}

