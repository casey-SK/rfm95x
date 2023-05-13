use rand::Rng;
use rppal::gpio::{Gpio, InputPin, OutputPin, Trigger, Level};
use rppal::spi::{Segment, Spi};
use std::error::Error;
use std::fmt::Display;
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Register {
	/** Taken from the LoRA register map (6.4, p. 102) in the RFM data sheet. Only accessible in LoRA mode */
	FIFO = 0x00,
	OpMode = 0x01,
	/* 0x02 - 0x05 RESERVED */
	FRFMSB = 0x06,
	FRFMID = 0x07,
	FRFLSB = 0x08,
	PAConfig = 0x09,
	PARamp = 0x0A,
	OverCurrentProtection = 0x0B,
	LNA = 0x0C,
	FIFOAddressPointer = 0x0D,
	FIFOTXBaseAddress = 0x0E,
	FIFORXBaseAddress = 0x0F,
	FIFORXCurrent = 0x10,
	IRQFlagsMask = 0x11, // Optional flags mask
	IRQFlags = 0x12,
	ReceiveNumberOfBytes = 0x13, // Number of payload bytes of latest packet received
	ReceiveValidHeaderCountMSB = 0x14, // Number of valid headers received since last transition into Rx mode. Header and packet counters are reseted in Sleep mode.
	ReceiveValidHeaderCountLSB = 0x15, // Number of valid headers received since last transition into Rx mode. Header and packet counters are reseted in Sleep mode.
	ReceiveValidPacketCountMSB = 0x16, // Number of valid packets received since last transition into Rx mode. Header and packet counters are reseted in Sleep mode.
	ReceiveValidPacketCountLSB = 0x17, // Number of valid packets received since last transition into Rx mode. Header and packet counters are reseted in Sleep mode.
	ModemStatus = 0x18, 
	LastSNRValue = 0x19, // Estimation of SNR on last packet received.In two’s compliment format mutiplied by 4.
	LastRSSIValue = 0x1A, // RSSI of the latest packet received (dBm)
	RSSIValue = 0x1B, // Current RSSI value (dBm)
	HopChannel = 0x1C, 
	ModemConfig1 = 0x1D,
	ModemConfig2 = 0x1E,
	SymbolTimeoutLSB = 0x1F,
	PreambleLengthMSB = 0x20,
	PreambleLengthLSB = 0x21, 
	PayloadLength = 0x22, // Payload length in bytes. The register needs to be set in implicit header mode for the expected packet length. A 0 value is not permitted
	MaxPayloadLength = 0x23, // Maximum payload length; if header payload length exceeds value a header CRC error is generated. Allows filtering of packet with a bad size.
	HopPeriod = 0x24, // Symbol periods between frequency hops. (0 = disabled). 1st hop always happen after the 1st header symbol
	FIFOReceiveAddress = 0x25, // Current value of RX databuffer pointer (address of last byte written by Lora receiver)
	ModemConfig3 = 0x26,

	// NodeAddress = 0x33,
	// ImageCalibration = 0x3B,
	Timer1Coefficient = 0x39,

	/** Taken from Table 85, available in either mode */
	DIOMapping1 = 0x40,
	DIOMapping2 = 0x41,
	Version = 0x42
}

bitflags! {
	struct Mode: u8 {
		// See p. 102, RegOpMode
		const SLEEP = 0b000;
		const STANDBY = 0b001;
		const FREQUENCY_SYNTHESIS_TRANSMIT = 0b010;
		const TRANSMIT = 0b011;
		const FREQUENCY_SYNTHESIS_RECEIVE = 0b100;
		const RECEIVE_CONTINUOUS = 0b101;
		const RECEIVE_SINGLE = 0b110;
		const CHANNEL_ACTIVITY_DETECTION = 0b111;

		const LORA = 0b1000_0000;
		const ACCESS_SHARED_REGISTERS = 0b0100_0000;
		const RESERVED_5 = 0b0010_0000;
		const RESERVED_4 = 0b0001_0000;
		const LOW_FREQUENCY_MODE = 0b0000_1000;
	}
}

bitflags! {
	struct ModemConfig3Flags: u8 {
		const IS_MOBILE_NODE = 0b0000_1000;
		const AUTO_AGC_ON = 0b0000_0100;
	}
}

bitflags! {
	// See page 106 of data sheet: RegModemConfig1
	struct ModemConfig1Flags: u8 {
		const BW7_8 = 0b0000_0000;
		const BW10_4 = 0b0001_0000;
		const BW15_6 = 0b0010_0000;
		const BW20_8 = 0b0011_0000;
		const BW31_25 = 0b0100_0000;
		const BW41_7 = 0b0101_0000;
		const BW62_5 = 0b0110_0000;
		const BW125 = 0b0111_0000;
		const BW250 = 0b1000_0000;
		const BW500 = 0b1001_0000;

		const CODING_RATE_4_5 = 0b0000_0010;
		const CODING_RATE_4_6 = 0b0000_0100;
		const CODING_RATE_4_7 = 0b0000_0110;
		const CODING_RATE_4_8 = 0b0000_1000;

		const IMPLICIT_HEADER_MODE_ON = 0b0000_0001;
	}
}

bitflags! {
	struct ModemConfig2Flags: u8 {
		const SF7 = 0x70;
		const SF8 = 0x80;
		const SF9 = 0x90;
		const SF10 = 0xA0;
		const SF11 = 0xB0;
		const SF12 = 0xC0;

		const TX_CONTINOUS_MODE_ON = 0b0000_1000; // Continuous mode, send multiple packets across the FIFO (used for spectral analysis)
		const RX_PAYLOAD_CRC_FOUND = 0b0000_0100; // CRC Information extracted from the received packet header

		const SYMBOL_TIMEOUT_MSB_1 = 0b0000_0010;
		const SYMBOL_TIMEOUT_MSB_0 = 0b0000_0001;
	}
}

bitflags! {
	struct IRQFlags: u8 {
		const CHANNEL_ACTIVITY_DETECTED = 0b0000_0001;
		const FHSS_CHANGE_CHANNEL = 0b0000_0010;
		const CHANNEL_ACTIVITY_DETECTION_DONE = 0b0000_0100;
		const TRANSMIT_DONE = 0b0000_1000;
		const VALID_HEADER_RECEIVED = 0b0001_0000;
		const PAYLOAD_CRC_ERROR = 0b0010_0000;
		const RECEIVE_DONE = 0b0100_0000;
		const RECEIVE_TIMEOUT = 0b1000_0000;
	}
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum DataRate {
	SF7_BW125,
	SF7_BW250,
	SF8_BW125,
	SF9_BW125,
	SF10_BW125,
	SF11_BW125,
	SF12_BW125,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Channel {
	Ch0,
	Ch1,
	Ch2,
	Ch3,
	Ch4,
	Ch5,
	Ch6,
	Ch7,
	Ch9,
	Multi,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Band {
	EU863,
	US901,
	AS920,
}

const RFM_VERSION: u8 = 0x12;

pub struct RFM95 {
	spi: Spi,
	irq_pin: InputPin,
	cs_bcm_pin: Option<u8>,
	tx_random_number: u8,
	tx_packets: u32,
	data_rate: DataRate,
	channel: Channel,
	band: Band,
	reset_pin: Option<InputPin>
}

pub struct ChipSelected {
	cs_pin: Option<OutputPin>,
}

#[derive(Debug)]
pub struct ModeChangeFailedErrorInfo {
	old_mode: u8,
	new_mode: Mode,
	set_mode: u8
}

#[derive(Debug)]
pub enum RFMError {
	InvalidVersion,
	ModeChangeFailed(ModeChangeFailedErrorInfo),
	TransmissionTimedOut
}

impl RFM95 {
	pub fn new(
		spi: Spi,
		irq_bcm_pin: u8,
		cs_bcm_pin: Option<u8>,
		data_rate: DataRate,
		band: Band,
		channel: Channel,
	) -> Result<RFM95, Box<dyn Error>> {
		let mut irq_pin = Gpio::new()?.get(irq_bcm_pin)?.into_input();
		irq_pin.set_interrupt(Trigger::RisingEdge)?;
		Ok(RFM95 {
			spi: spi,
			irq_pin: irq_pin,
			cs_bcm_pin: cs_bcm_pin,
			tx_random_number: 0,
			tx_packets: 0,
			data_rate: data_rate,
			band: band,
			channel: channel,
			reset_pin: None
		})
	}

	/** Set mode of the RFM9x chip and verify it was set correctly */
	fn set_mode(&mut self, mode: Mode) -> Result<(), Box<dyn Error>> {
		let old_mode_raw = self.read_register(Register::OpMode)?;
		let old_mode = Mode::from_bits(old_mode_raw);

		// No need to change modes if the last mode is equal to the current one
		if old_mode == Some(mode) {
			return Ok(());
		}
		// println!("Set mode {:?} => {:?}", old_mode, mode);
		self.write_register(Register::OpMode, mode.bits)?;
		thread::sleep(Duration::from_millis(10));

		// Check the correct mode was set
		let set_mode_raw = self.read_register(Register::OpMode)?;
		if set_mode_raw != mode.bits {
			return Err(Box::new(RFMError::ModeChangeFailed(ModeChangeFailedErrorInfo {
				old_mode: old_mode_raw,
				new_mode: mode,
				set_mode: set_mode_raw
			})));
		}
		Ok(())
	}

	pub fn reset(&mut self, bcm_pin: u8) -> Result<(), Box<dyn Error>> {
		/* Cycle reset pin. This is a bit weird: first, we set it to output low, then the pin is changed to be a pull-up input.
		According to the Python version of the RFM95 driver, this is the only way it will actually work. */
		self.reset_pin = None;
		{
			let mut pin = Gpio::new()?.get(bcm_pin)?.into_output();
			pin.set_low();
			thread::sleep(Duration::from_millis(1));
		}
		let in_pin = Gpio::new()?.get(bcm_pin)?.into_input_pullup();
		thread::sleep(Duration::from_millis(500));
		self.reset_pin = Some(in_pin);

		// Check version
		if self.get_version()? != RFM_VERSION {
			return Err(Box::new(RFMError::InvalidVersion));
		}

		// Set modes
		self.set_mode(Mode::SLEEP)?;

		// Go to LoRA mode
		self.set_mode(Mode::SLEEP | Mode::LORA)?;

		// PA pin (maximum power, 17 dBm)
		self.write_register(Register::PAConfig, 0xFF)?;

		// Rx Timeout set to 37 symbols
		self.write_register(Register::SymbolTimeoutLSB, 0x25)?;

		// Preamble length set to 8 symbols
		// 0x0008 + 4 = 12
		self.write_register(Register::PreambleLengthMSB, 0x00)?;
		self.write_register(Register::PreambleLengthLSB, 0x08)?;

		// LoRA sync word
		self.write_register(Register::Timer1Coefficient, 0x34)?;

		// IQ
		// self.write_register(Register::NodeAddress, 0x27)?;
		// self.write_register(Register::ImageCalibration, 0x1D)?;

		// FIFO pointers
		self.write_register(Register::FIFOTXBaseAddress, 0x80)?;
		self.write_register(Register::FIFORXBaseAddress, 0x00)?;

		self.tx_random_number = 0;
		Ok(())
	}

	fn read_register(&mut self, register: Register) -> Result<u8, Box<dyn Error>> {
		let s = self.select();
		let cmd = (register as u8) & 0x7F;
		let mut buffer = [42u8; 1];
		self.spi
			.transfer_segments(&[Segment::with_write(&[cmd]), Segment::with_read(&mut buffer)])?;
		let r = Ok(buffer[0]);
		drop(s);
		r
	}

	fn write_register(&mut self, register: Register, value: u8) -> Result<(), Box<dyn Error>> {
		//println!("REG {:02x} = {:8b} ({:02x}) {:?}", register as u8, value, value, register);
		let s = self.select();
		let cmd = (register as u8) | 0x80;
		self.spi.transfer_segments(&[Segment::with_write(&[cmd, value])])?;
		drop(s);
		Ok(())
	}

	fn wait_for_interrupt(&mut self, timeout: Duration) -> Result<bool, Box<dyn Error>> {
		self.write_register(Register::IRQFlags, 0xFF)?; // Clear IRQ flags
		assert_eq!(self.irq_pin.is_high(), false);
		let result = match self.irq_pin.poll_interrupt(true, Some(timeout))? {
			Some(Level::Low) => false, // Should not happen?
			Some(Level::High) => true,
			None => false
		};

		let irq_flags = IRQFlags::from_bits(self.read_register(Register::IRQFlags)?);
		println!("IRQ status: fired {}, pin {}, flags={:?}", result, self.irq_pin.is_high(), irq_flags);
		Ok(result)
	}

	/** Receive packet in receive window after transmit (re-use set radio parameters).
	 * Note: there are two receive windows:
	 * - RX1 opens a second after transmission (on the same frequency as the uplink)
	 * - RX2 opens two seconds after transmission (on a fixed, configurable frequency; the default is 869.525 MHz)
	 *
	 * The data rates used:
	 * - RX1 uses the data rate of the uplink, unless an offset has been configured (see LoRAWAN regional spec. 2.2.7)
	 * - RX2 again is fixed and configurable; the default is SF12, 125 kHz.
	 */
	pub fn receive_packet(&mut self, channel: Channel, data_rate: DataRate, with_crc: bool, timeout: Duration) -> Result<[u8; 255], Box<dyn Error>> {
		
		let mut buffer = [0 as u8; 255];
		
		self.set_mode(Mode::LORA | Mode::STANDBY)?;

		// Put receiver in receive mode
		self.set_mode(Mode::LORA | Mode::RECEIVE_CONTINUOUS)?;
		self.set_frequency(channel)?;
		self.set_data_rate(data_rate, with_crc)?;
		self.write_register(Register::PayloadLength, 0u8)?;

		// Set IRQ pin to become high when a message has been received (RxDone)
		self.write_register(Register::DIOMapping1, 0x00)?;

		println!("Before RX: {} bytes, {} pkts, {} headers, last RSSI={}", self.read_register(Register::ReceiveNumberOfBytes)?, self.read_register(Register::ReceiveValidPacketCountLSB)?, self.read_register(Register::ReceiveValidHeaderCountLSB)?, self.read_register(Register::LastRSSIValue)?);

		// Wait for the interrupt pin to become high
		self.wait_for_interrupt(timeout)?;
		println!("RX: {} bytes, {} pkts, {} headers, last RSSI={}", self.read_register(Register::ReceiveNumberOfBytes)?, self.read_register(Register::ReceiveValidPacketCountLSB)?, self.read_register(Register::ReceiveValidHeaderCountLSB)?, self.read_register(Register::LastRSSIValue)?);
		let size = self.read_register(Register::ReceiveNumberOfBytes)?;
		let fifo_addr = self.read_register(Register::FIFORXCurrent)?;
        self.write_register(Register::FIFOAddressPointer, fifo_addr)?;
        for i in 0..size {
            let byte = self.read_register(Register::FIFO)?;
            buffer[i as usize] = byte;
        }
        self.write_register(Register::FIFOAddressPointer, 0)?;


		// Put transceiver to sleep again
		self.set_mode(Mode::LORA | Mode::STANDBY)?;
		Ok(buffer)
	}

	pub fn receive_packet_on_tx(&mut self, with_crc: bool, timeout: Duration) -> Result<[u8; 255], Box<dyn Error>> {
		self.receive_packet(self.channel, self.data_rate, with_crc, timeout)
	}

	fn set_frequency(&mut self, channel: Channel) -> Result<(), Box<dyn Error>> {
		let frequency = channel.frequency(&self.band);
		self.write_register(Register::FRFMSB, frequency[0])?;
		self.write_register(Register::FRFMID, frequency[1])?;
		self.write_register(Register::FRFLSB, frequency[2])?;
		println!("Frequency set to {:?} {:02x?}", channel, frequency);
		Ok(())
	}

	fn set_data_rate(&mut self, data_rate: DataRate, enable_crc: bool) -> Result<(), Box<dyn Error>> {
		let mut modem_config_2 = data_rate.modem_config_2();

		if enable_crc {
			modem_config_2 |= ModemConfig2Flags::RX_PAYLOAD_CRC_FOUND;
		}

		self.write_register(Register::ModemConfig2, modem_config_2.bits)?;
		self.write_register(Register::ModemConfig1, data_rate.modem_config_1().bits)?;
		self.write_register(Register::ModemConfig3, data_rate.modem_config_3().bits)?;
		Ok(())
	}

	pub fn send_packet(&mut self, packet: &[u8]) -> Result<(), Box<dyn Error>> {
		assert!(packet.len() > 0);
		assert!(packet.len() < 255);

		self.set_mode(Mode::LORA | Mode::STANDBY)?;

		// Configure DIO0 as the IRQ pin to become low when TxDone
		self.write_register(Register::DIOMapping1, 0x40)?;

		// Set channel
		self.set_frequency(self.channel)?;

		// Set data rate
		self.set_data_rate(self.data_rate, true)?;

		// Set payload length
		self.write_register(Register::PayloadLength, packet.len() as u8)?;

		// Set FIFO pointer to start of transmit part in FIFO
		self.write_register(Register::FIFOAddressPointer, 0x80)?;

		// Write payload to FIFO
		for i in 0..packet.len() {
			self.write_register(Register::FIFO, packet[i])?;
			self.tx_packets += 1;
		}

		// Switch to transmit mode
		self.set_mode(Mode::LORA | Mode::TRANSMIT)?;

		// Wait for the interrupt pin to become high
		if !self.wait_for_interrupt(Duration::from_millis(1000))? {
			return Err(Box::new(RFMError::TransmissionTimedOut));
		}

		// Put transceiver to standby again
		self.set_mode(Mode::LORA | Mode::STANDBY)?;
		Ok(())
	}

	fn select(&mut self) -> Result<ChipSelected, Box<dyn Error>> {
		ChipSelected::new(self.cs_bcm_pin)
	}

	pub fn get_version(&mut self) -> Result<u8, Box<dyn Error>> {
		self.read_register(Register::Version)
	}
}

impl Display for RFMError {
	fn fmt(&self, out: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
		write!(
			out,
			"{}",
			match self {
				RFMError::InvalidVersion => String::from("invalid version"),
				RFMError::ModeChangeFailed(info) => format!("mode change failed: {:?}", info),
				RFMError::TransmissionTimedOut => String::from("transmission timed out")
			}
		)
	}
}

impl Error for RFMError {}

impl ChipSelected {
	pub fn new(cs_bcm_pin: Option<u8>) -> Result<ChipSelected, Box<dyn Error>> {
		Ok(ChipSelected {
			cs_pin: match cs_bcm_pin {
				Some(cs_pin) => {
					let mut p = Gpio::new()?.get(cs_pin)?.into_output();
					p.set_low();
					Some(p)
				}
				None => None,
			},
		})
	}
}

impl Drop for ChipSelected {
	fn drop(&mut self) {
		match self.cs_pin {
			Some(ref mut pin) => pin.set_high(),
			None => {}
		}
	}
}

impl DataRate {
	fn modem_config_1(&self) -> ModemConfig1Flags {
		match self {
			DataRate::SF7_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_5,
			DataRate::SF7_BW250 => ModemConfig1Flags::BW250 | ModemConfig1Flags::CODING_RATE_4_5,
			DataRate::SF8_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_5,
			DataRate::SF9_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_5,
			DataRate::SF10_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_5,
			DataRate::SF11_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_8,
			DataRate::SF12_BW125 => ModemConfig1Flags::BW125 | ModemConfig1Flags::CODING_RATE_4_8,
		}
	}

	fn modem_config_2(&self) -> ModemConfig2Flags {
		match self {
			DataRate::SF7_BW125 => ModemConfig2Flags::SF7,
			DataRate::SF7_BW250 => ModemConfig2Flags::SF7,
			DataRate::SF8_BW125 => ModemConfig2Flags::SF8,
			DataRate::SF9_BW125 => ModemConfig2Flags::SF9,
			DataRate::SF10_BW125 => ModemConfig2Flags::SF10,
			DataRate::SF11_BW125 => ModemConfig2Flags::SF11,
			DataRate::SF12_BW125 => ModemConfig2Flags::SF12,
		}
	}

	fn modem_config_3(&self) -> ModemConfig3Flags {
		match self {
			DataRate::SF7_BW125 => ModemConfig3Flags::AUTO_AGC_ON,
			DataRate::SF7_BW250 => ModemConfig3Flags::AUTO_AGC_ON,
			DataRate::SF8_BW125 => ModemConfig3Flags::AUTO_AGC_ON,
			DataRate::SF9_BW125 => ModemConfig3Flags::AUTO_AGC_ON,
			DataRate::SF10_BW125 => ModemConfig3Flags::AUTO_AGC_ON,
			DataRate::SF11_BW125 => ModemConfig3Flags::AUTO_AGC_ON | ModemConfig3Flags::IS_MOBILE_NODE,
			DataRate::SF12_BW125 => ModemConfig3Flags::AUTO_AGC_ON | ModemConfig3Flags::IS_MOBILE_NODE,
		}
	}
}

impl Channel {
	fn frequency(&self, band: &Band) -> [u8; 3] {
		match band {
			Band::EU863 => match self {
				Channel::Ch0 => [0xD9, 0x06, 0x8B], //Channel 0 868.100 MHz / 61.035 Hz = 14222987 = 0xD9068B
				Channel::Ch1 => [0xD9, 0x13, 0x58], //Channel 1 868.300 MHz / 61.035 Hz = 14226264 = 0xD91358
				Channel::Ch2 => [0xD9, 0x20, 0x24], //Channel 2 868.500 MHz / 61.035 Hz = 14229540 = 0xD92024
				Channel::Ch3 => [0xD8, 0xC6, 0x8B], //Channel 3 867.100 MHz / 61.035 Hz = 14206603 = 0xD8C68B
				Channel::Ch4 => [0xD8, 0xD3, 0x58], //Channel 4 867.300 MHz / 61.035 Hz = 14209880 = 0xD8D358
				Channel::Ch5 => [0xD8, 0xE0, 0x24], //Channel 5 867.500 MHz / 61.035 Hz = 14213156 = 0xD8E024
				Channel::Ch6 => [0xD8, 0xEC, 0xF1], //Channel 6 867.700 MHz / 61.035 Hz = 14216433 = 0xD8ECF1
				Channel::Ch7 => [0xD8, 0xF9, 0xBE], //Channel 7 867.900 MHz / 61.035 Hz = 14219710 = 0xD8F9BE
				Channel::Multi => Channel::random().frequency(band),
				Channel::Ch9 => [0xD9, 0x61, 0xBE], // Channel 9 869.525 MHz / 61.035 Hz = 14246334 = 0xD961BE
			},

			Band::US901 => match self {
				Channel::Ch0 => [0xE1, 0xF9, 0xC0], //Channel 0 903.900 MHz / 61.035 Hz = 14809536 = 0xE1F9C0
				Channel::Ch1 => [0xE2, 0x06, 0x8C], //Channel 1 904.100 MHz / 61.035 Hz = 14812812 = 0xE2068C
				Channel::Ch2 => [0xE2, 0x13, 0x59], //Channel 2 904.300 MHz / 61.035 Hz = 14816089 = 0xE21359
				Channel::Ch3 => [0xE2, 0x20, 0x26], //Channel 3 904.500 MHz / 61.035 Hz = 14819366 = 0xE22026
				Channel::Ch4 => [0xE2, 0x2C, 0xF3], //Channel 4 904.700 MHz / 61.035 Hz = 14822643 = 0xE22CF3
				Channel::Ch5 => [0xE2, 0x39, 0xC0], //Channel 5 904.900 MHz / 61.035 Hz = 14825920 = 0xE239C0
				Channel::Ch6 => [0xE2, 0x46, 0x8C], //Channel 6 905.100 MHz / 61.035 Hz = 14829196 = 0xE2468C
				Channel::Ch7 => [0xE2, 0x53, 0x59], //Channel 7 905.300 MHz / 61.035 Hz = 14832473 = 0xE25359
				Channel::Multi => Channel::random().frequency(band),
				Channel::Ch9 => unreachable!()
			},

			Band::AS920 => match self {
				Channel::Ch0 => [0xE6, 0xCC, 0xF4], //Channel 0 868.100 MHz / 61.035 Hz = 15125748 = 0xE6CCF4
				Channel::Ch1 => [0xE6, 0xD9, 0xC0], //Channel 1 868.300 MHz / 61.035 Hz = 15129024 = 0xE6D9C0
				Channel::Ch2 => [0xE6, 0x8C, 0xF3], //Channel 2 868.500 MHz / 61.035 Hz = 15109363 = 0xE68CF3
				Channel::Ch3 => [0xE6, 0x99, 0xC0], //Channel 3 867.100 MHz / 61.035 Hz = 15112640 = 0xE699C0
				Channel::Ch4 => [0xE6, 0xA6, 0x8D], //Channel 4 867.300 MHz / 61.035 Hz = 15115917 = 0xE6A68D
				Channel::Ch5 => [0xE6, 0xB3, 0x5A], //Channel 5 867.500 MHz / 61.035 Hz = 15119194 = 0xE6B35A
				Channel::Ch6 => [0xE6, 0xC0, 0x27], //Channel 6 867.700 MHz / 61.035 Hz = 15122471 = 0xE6C027
				Channel::Ch7 => [0xE6, 0x80, 0x27], //Channel 7 867.900 MHz / 61.035 Hz = 15106087 = 0xE68027
				Channel::Multi => Channel::random().frequency(band),
				Channel::Ch9 => unreachable!()
			},
		}
	}

	fn random() -> Channel {
		let mut rng = rand::thread_rng();
		// Only use channels 0..=7
		match rng.gen_range::<u8, u8, u8>(0, 8) {
			0 => Channel::Ch0,
			1 => Channel::Ch1,
			2 => Channel::Ch2,
			3 => Channel::Ch3,
			4 => Channel::Ch4,
			5 => Channel::Ch5,
			6 => Channel::Ch6,
			7 => Channel::Ch7,
			_ => unreachable!(),
		}
	}
}
