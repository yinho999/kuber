use clap::Parser;
use kube::{Error};
use kuber::Cli;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let app = kuber::App::new().await?;
    cli.run(app).await?;
    Ok(())
}
