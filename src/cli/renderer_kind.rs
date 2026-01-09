use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum RendererKind {
    Human,
    Diff,
}


