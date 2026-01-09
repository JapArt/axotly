use clap::{Parser, ArgGroup};
use crate::cli::RendererKind;

#[derive(Parser, Debug)]
#[command(
    name = "axotly",
    about = "Minimal API testing tool",
    group(
        ArgGroup::new("input")
            .required(true)
            .args(["file", "url"])
    )
)]
pub struct Cli {
    /// Run requests from a .http file
    #[arg(short, long)]
    pub file: Option<String>,

    #[arg(short, long, default_value = "human", requires = "file")]
    pub renderer: RendererKind,
    
    /// Number of concurrent requests (min: 1, default: CPU cores)
    #[arg(
        short, 
        long, 
        requires = "file",
        default_value_t = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        value_parser = clap::builder::RangedI64ValueParser::<usize>::new().range(1..200)
    )]
    pub concurrently: usize,

    /// Run file requests concurrently (parallel)
    #[arg(long, requires = "file")]
    pub async_mode: bool,

    /// The URL to fetch (ignored if --file is used)
    #[arg(short, long, default_value = "http://httpbin.org/get")]
    pub url: String,

    /// HTTP method (get, post, put, delete, patch)
    #[arg(short, long, default_value = "get")]
    pub method: String,

    /// Request body
    #[arg(short, long, conflicts_with = "file")]
    pub body: Option<String>,
}
