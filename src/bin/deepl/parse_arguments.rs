pub use clap::Parser;
pub use clap::Subcommand;

/// Command line client for the DeepL API.
#[derive(Parser, Debug)]
#[clap(name = "deepl",version(env!("CARGO_PKG_VERSION")),setting(clap::AppSettings::PropagateVersion))]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubCmd {
    Translate(Translate),
    /// Fetch imformation about account usage & limits.
    UsageInformation,
    /// Fetch list of available source and target languages.
    Languages,
}

/// A subcommand for controlling testing
#[derive(Parser, Debug)]
pub struct Translate {
    /// Source language (optional)
    #[clap(long)]
    pub source_language: Option<String>,
    /// Target language (required)
    #[clap(long)]
    pub target_language: String,
    /// Input filepath (optional, reads from STDIN by default)
    #[clap(long)]
    pub input_file: Option<String>,
    /// Output filepath (optional, prints to STDOUT by default)
    #[clap(long)]
    pub output_file: Option<String>,

    /// Preserve formatting
    #[clap(long)]
    pub preserve_formatting: bool,
    /// Increase formality
    #[clap(long)]
    pub formality_more: bool,
    /// Decrease formality
    #[clap(long)]
    pub formality_less: bool,
}
