pub use clap::Clap;

/// Command line client for the DeepL API.
#[derive(Clap)]
#[clap(name = "deepl",version(env!("CARGO_PKG_VERSION")),setting(clap::AppSettings::GlobalVersion))]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Translate(Translate),
    /// Fetch imformation about account usage & limits.
    UsageInformation,
    /// Fetch list of available source and target languages.
    Languages,
}

/// A subcommand for controlling testing
#[derive(Clap)]
pub struct Translate {
    /// Source language (optional)
    #[clap(long)]
    pub source_language: Option<String>,
    /// Target language (required)
    #[clap(long)]
    pub target_language: String,
    /// Input filepath (optional, reads from STDIN by default)
    #[clap(long)]
    pub input_filepath: Option<String>,
    /// Output filepath (optional, prints to STDOUT by default)
    #[clap(long)]
    pub output_filepath: Option<String>,

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
