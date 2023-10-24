use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::time::Duration;

use rfm9x::{Band, Channel, DataRate, RFM95};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Ping(PingArgs),

    Pong(PongArgs),
}

#[derive(Args)]
struct PingArgs {
    /// The number of pings to send out before closing the program.
    #[arg(short, long, default_value_t = 1)]
    count: u8,

    /// the time in seconds between pings. If set to zero, it will only ping when a pong has been recieved (except the first one).
    #[arg(short, long, default_value_t = 0)]
    delay: u32,
}

#[derive(Args)]
struct PongArgs {
    #[arg(short, long, default_value_t = 120)]
    /// The amount of time the program will listen for a ping. Resets after each recieved ping.
    time: u32,
}

fn ping(count: u8, delay: u32) {
    let spi: rppal::spi::Spi =
        Spi::new(Bus::Spi0, SlaveSelect::Ss1, 4_000_000, Mode::Mode0).unwrap();

    let mut rfm = RFM95::new(
        spi,
        22,
        Some(7),
        DataRate::SF7_BW125,
        Band::US901,
        Channel::Ch3,
    )
    .unwrap();

    rfm.reset(25).unwrap();
    let message = "PING";

    rfm.send_packet(message.as_bytes()).unwrap(); // Raw packet
}

fn pong(timeout: u32) {
    unimplemented!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Ping(ping) => {
            unimplemented!()
        }
        Commands::Pong(pong) => {
            unimplemented!()
        }
    }
}
