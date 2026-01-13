use clap::{Parser, ArgGroup};
use crate::cli::RendererKind;

#[derive(Parser, Debug)]
#[command(
    name = "axotly",
    about = "Fast, reliable, and expressive API testing — designed for developer happiness.",
    group(
        ArgGroup::new("input")
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
        default_value_t = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
        value_parser = clap::builder::RangedI64ValueParser::<usize>::new().range(1..200)
    )]
    pub concurrently: usize,

    /// Show responses 
    #[arg(long, requires="file")]
    pub show_response: bool,

    /// URL to fetch (positional, curl-style)
    #[arg(
        value_name = "URL",
        default_value = "http://httpbin.org/get",
        conflicts_with = "file"
    )]
    pub url: Option<String>,

    /// HTTP method (get, post, put, delete, patch)
    #[arg(short, long, default_value = "GET")]
    pub method: String,

    /// Body for request
    #[arg(short, long)]
    pub body: Option<String>,

    /// Json body request
    #[arg(short = 'j', long)]
    pub json: Option<String>,

}

