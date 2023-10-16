use rfm9x::{Band, Channel, DataRate, RFM95};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

fn main() {
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

    let message = "Hello, World!";

    rfm.send_packet(message.as_bytes()).unwrap(); // Raw packet
}
