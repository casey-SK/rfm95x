use clap::{Args, Parser, Subcommand};
use std::{error::Error, thread::sleep};
use std::time::{Duration, Instant};
use chrono::prelude::*;

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
    delay: u64,

    /// The amount of time the program will listen for a ping. Resets after each recieved ping.
    #[arg(short, long, default_value_t = 120)]
    timeout: u64,
}

#[derive(Args)]
struct PongArgs {
    #[arg(short, long, default_value_t = 120)]
    /// The amount of time the program will listen for a ping. Resets after each recieved ping.
    timeout: u64,
}



fn setup_radio() -> Result<RFM95, Box<dyn Error>> {
    // Define spi
    let spi: rppal::spi::Spi =
        Spi::new(Bus::Spi0, SlaveSelect::Ss1, 4_000_000, Mode::Mode0)?;

    // Define radio
    let mut rfm = RFM95::new(
        spi,
        22,
        Some(7),
        DataRate::SF7_BW125,
        Band::US901,
        Channel::Ch3,
    )
    ?;

    rfm.reset(25)?;

    // recieve a packet (unused, not sure why but it needs to happen.)
    let (_pkt, _size) = rfm.receive_packet(
        Channel::Ch3,
        DataRate::SF7_BW125,
        false,
        Duration::from_secs(1),
    ).unwrap();
    
    return Ok(rfm);
}


fn send_it(rfm: &mut RFM95, m: &str) -> Result<(), Box<dyn Error>> {

    let msg = format!("TX: [{}] - {}", Utc::now().to_string(), m);
    println!("{}", msg);
    rfm.send_packet(msg.as_bytes()).unwrap();
    Ok(())
}

fn get_it(rfm: &mut RFM95, timeout: u64) -> Result<String, Box<dyn Error>> {
    let (pkt, _size) = rfm.receive_packet(
        Channel::Ch3,
        DataRate::SF7_BW125,
        false,
        Duration::from_secs(timeout + 1),
    ).unwrap();

    let msg = String::from_utf8_lossy(&pkt);
    let msg2 = msg.strip_suffix(0 as char).unwrap();

    return Ok(format!("RX: [{}] [RSSI: {}] [SNR: {}]- {}", Utc::now().to_string(), rfm.get_rssi().unwrap(),rfm.get_rssi().unwrap(), msg2));
}

fn ping(count: u8, delay: u64, timeout: u64) -> Result<(), Box<dyn Error>> {
    
    // setup radio
    let mut rfm = setup_radio().unwrap();

    // send first packet
    send_it(&mut rfm, "PING")?;

    let mut counter: u8 = 1;
    let mut now = Instant::now();
    while counter < count {
        let m = get_it(&mut rfm, timeout)?;
        if now.elapsed().as_secs() >= timeout.into() {
            println!("\nTIMEOUT\n");
            break;
        }
        println!("{}", m);
        sleep(Duration::from_secs(delay));
        send_it(&mut rfm, "PING")?;
        now = Instant::now();
        counter += 1;
    }

    Ok(())
}

fn pong(timeout: u64) -> Result<(), Box<dyn Error>> {
    
    let mut rfm = setup_radio()?;

    let mut now = Instant::now();
    loop {
        let m = get_it(&mut rfm, timeout)?;
        if now.elapsed().as_secs() >= timeout.into() {
            println!("\nTIMEOUT\n");
            break;
        }
        println!("{}", m);
        send_it(&mut rfm, "PONG")?;
        now = Instant::now();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Ping(args) => {
            ping(args.count, args.delay, args.timeout)?;
        }
        Commands::Pong(args) => {
            pong(args.timeout)?;
        }
    }

    Ok(())
}
