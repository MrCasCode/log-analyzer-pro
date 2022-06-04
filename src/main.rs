
use std::error::Error;


use terminal_ui::async_main;


use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Settings file containing formats, filters or color customization
    #[clap(short, long)]
    settings: Option<String>,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    async_std::task::block_on(async_main(args.settings))?;

    Ok(())
}
