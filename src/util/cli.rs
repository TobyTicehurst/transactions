use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub csv_filepath: String,
}

impl Cli {
    pub fn from_args() -> Self {
        Cli::parse()
    }
}
