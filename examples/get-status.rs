use clap::Parser;
use thermosmart::Thermostat;

#[derive(Parser)]
struct Args {
    endpoint: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let thermostat = Thermostat::new(&args.endpoint);
    println!("status: {:?}", thermostat.get_status().await);
}
