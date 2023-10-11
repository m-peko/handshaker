use clap::Parser;

mod cli;

#[tokio::main]
async fn main() {
    let _args = cli::Arguments::parse();
}
