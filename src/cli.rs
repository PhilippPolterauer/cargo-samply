use clap::Parser;

/// A cargo subcommand for profiling binaries using samply
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Trailing arguments passed to the binary being profiled
    #[arg(name = "TRAILING_ARGUMENTS")]
    pub args: Vec<String>,

    /// Build with the specified profile
    #[arg(short, long, default_value = "samply")]
    pub profile: String,

    /// Binary to run
    #[arg(short, long)]
    pub bin: Option<String>,

    /// Example to run
    #[arg(short, long)]
    pub example: Option<String>,

    /// Build features to enable
    #[arg(short, long)]
    pub features: Option<String>,

    /// Disable default features
    #[arg(long)]
    pub no_default_features: bool,

    /// Print extra output to help debug problems
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
