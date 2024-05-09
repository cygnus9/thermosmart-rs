use std::error::Error;

use clap::Parser;
use thermosmart::Thermostat;

#[derive(Parser)]
struct Args {
    endpoint: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let thermostat = Thermostat::create(&args.endpoint)?;
    println!("status: {:?}", thermostat.get_status().await?);

    Ok(())
}
