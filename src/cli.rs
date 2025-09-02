use clap::Parser;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
pub enum CargoCli {
    Samply(Config),
}

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

/// A cargo subcommand for profiling binaries using samply
#[derive(clap::Args)]
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
    pub features: Vec<String>,

    /// Disable default features
    #[arg(long)]
    pub no_default_features: bool,

    /// Print extra output to help debug problems
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Disable the automatic samply start
    #[arg(short, long, default_value_t = false)]
    pub no_samply: bool,
}
