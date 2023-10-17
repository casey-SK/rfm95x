use std::error::Error;
use std::time::Duration;

use rfm9x::{Band, Channel, DataRate, RFM95};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

fn main() -> Result<(), Box<dyn Error>> {
    //println!("debug 0");

    let spi: rppal::spi::Spi =
        Spi::new(Bus::Spi0, SlaveSelect::Ss1, 4_000_000, Mode::Mode0).unwrap();

    let mut rfm = RFM95::new(
        spi,
        22,
        Some(7),
        DataRate::SF7_BW125,
        Band::US901,
        Channel::Ch3,
    )?;
    rfm.reset(25)?;

    //println!("debug 00");

    loop {
        //println!("debug 000");
        let pkt = rfm.receive_packet(
            Channel::Ch3,
            DataRate::SF7_BW125,
            false,
            Duration::from_secs(30),
        )?;
        let msg = String::from_utf8_lossy(&pkt);
        let msg2 = msg.strip_suffix(0 as char).unwrap();

        println!("{}", msg2);
    }
}
