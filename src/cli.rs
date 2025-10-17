use clap::{command, Parser};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub directory: Option<String>,
}
