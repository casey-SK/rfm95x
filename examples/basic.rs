use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::error::Error;
use rfm9x::{Band, Channel, DataRate, RFM95};

fn main() -> Result<(), Box<dyn Error>> {
	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)?;

	let mut rfm = RFM95::new(
		spi,
		5, // IRQ pin
		Some(8), // CS pin
		DataRate::SF7_BW125,
		Band::EU863,
		Channel::Ch3,
	)?;
	rfm.reset(25)?;
	rfm.send_packet("Hello world!".as_bytes())?; // Raw packet
	Ok(())
}
