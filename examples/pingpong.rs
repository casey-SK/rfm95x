use clap::{Args, Parser, Subcommand};
use std::{error::Error, thread::sleep};
use std::time::{Duration, Instant};
use chrono::prelude::*;

// use std::{thread, time};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use linux_embedded_hal::I2cdev;


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
    #[arg(short, long, default_value_t = 10)]
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
        DataRate::SF12_BW125,
        Band::US901,
        Channel::Ch3,
    )
    ?;

    rfm.reset(25)?;

    // recieve a packet (unused, not sure why but it needs to happen.)
    let (_pkt, _size) = rfm.receive_packet(
        Channel::Ch3,
        DataRate::SF12_BW125,
        false,
        Duration::from_secs(1),
    )?;
    
    return Ok(rfm);
}


fn send_it(rfm: &mut RFM95, m: &str) -> Result<(), Box<dyn Error>> {

    let msg = format!("{} {}", Utc::now().round_subsecs(2).time().to_string(), m);
    println!("TX: {}", msg);
    rfm.send_packet(msg.as_bytes())?;
    Ok(())
}

fn get_it(rfm: &mut RFM95, timeout: u64) -> Result<String, Box<dyn Error>> {

    let mut t:u64 = 120;

    if timeout ==0 { 
        t = 120;
    } else {
        t = timeout;
    }

    let (pkt, _size) = rfm.receive_packet(
        Channel::Ch3,
        DataRate::SF12_BW125,
        false,
        Duration::from_secs(t + 1),
    )?;

    let msg = String::from_utf8_lossy(&pkt);
    let msg2 = msg.strip_suffix(0 as char).unwrap().to_string();

    return Ok(msg2);
}

fn print_rx_oled(line1: String, line2: String) {

    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    
    let interface = I2CDisplayInterface::new(i2c);
    
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline(&line1, Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline(&line2, Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();
}

fn ping(count: u8, delay: u64, timeout: u64) -> Result<(), Box<dyn Error>> {
    
    let oled_print_l1 = "PING".to_string();
    let oled_print_l2 = "ABOUT TO TX".to_string();
    print_rx_oled(oled_print_l1, oled_print_l2);

    // setup radio
    let mut rfm = setup_radio()?;
    let mut counter: u8 = 1;
    // send first packet
    let mut fm = format!("{}/{} - PING", counter, count);
    send_it(&mut rfm, &fm)?;

    
    let mut now = Instant::now();
    while counter < count {
        let m = get_it(&mut rfm, delay)?;
        if (now.elapsed().as_secs() >= timeout.into() && timeout !=0) {
            println!("\nTIMEOUT\n");
            let oled_print_l1 = "PING - TIMEOUT".to_string();
            print_rx_oled(oled_print_l1, "".to_string());
            break;
        }
        println!("RX: [{}] [RSSI: {}] [SNR: {}]- {}", Utc::now().round_subsecs(2).time().to_string(), rfm.get_rssi().unwrap(),rfm.get_rssi().unwrap(), m);
        sleep(Duration::from_secs(delay));
        counter += 1;
        fm = format!("{}/{} - PING", counter, count);
        print_rx_oled(fm.clone(), "".to_string());
        send_it(&mut rfm, &fm)?;
        now = Instant::now();
        
    }

    let oled_print_l1 = "PING - DONE".to_string();
    print_rx_oled(oled_print_l1, "".to_string());

    Ok(())
}



fn pong(timeout: u64) -> Result<(), Box<dyn Error>> {
    
    let mut rfm = setup_radio()?;

    let oled_print_l1 = "PONG".to_string();
    let oled_print_l2 = "WAITING FOR RX".to_string();
    print_rx_oled(oled_print_l1, oled_print_l2);


    let mut now = Instant::now();
    loop {
        let m = get_it(&mut rfm, timeout)?;
        if now.elapsed().as_secs() >= timeout.into() && timeout !=0{
            println!("\nTIMEOUT\n");
            let oled_print_l1 = "PONG - TIMEOUT".to_string();
            print_rx_oled(oled_print_l1, "".to_string());
            break;
        }
        println!("RX: [{}] [RSSI: {}] [SNR: {}]- {}", Utc::now().round_subsecs(2).time().to_string(), rfm.get_rssi().unwrap(),rfm.get_rssi().unwrap(), m);
        let (t1, b1) = m.split_at(12);
        let t2 = t1.trim_matches(char::from(0)).trim().to_string();
        let b2 = b1.trim_matches(char::from(0)).trim().to_string();
        print_rx_oled(t2, b2);
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
