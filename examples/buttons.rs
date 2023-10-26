use std::process::Command;


use std::error::Error;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::Level;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_BUTTON_LEFT: u8 = 5;
const GPIO_BUTTON_CENTRE: u8 = 6;
const GPIO_BUTTON_RIGHT: u8 = 12;


type Callback = fn(Level);

fn ping(_lvl: Level){
    println!("starting ping");
    Command::new("/home/casey/rfm95x/pingpong")
        .args(["ping", "--count=6", "--delay=10 --timeout=0"])
        .spawn()
        .expect("failed to start external executable");
}

fn pong(_lvl: Level){
    Command::new("/home/casey/rfm95x/pingpong")
        .args(["pong", "--timeout=0"])
        .spawn()
        .expect("failed to start external executable");
}

fn nothing_here(_lvl: Level){

}


fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin1 = Gpio::new()?.get(GPIO_BUTTON_LEFT)?.into_input_pullup();
    let mut pin2 = Gpio::new()?.get(GPIO_BUTTON_CENTRE)?.into_input_pullup();
    let mut pin3 = Gpio::new()?.get(GPIO_BUTTON_RIGHT)?.into_input_pullup();

    let ping: Callback = ping;
    let pong: Callback = pong;
    let nothin: Callback = nothing_here;
    
    pin1.set_async_interrupt(Trigger::FallingEdge, ping).unwrap();
    pin2.set_async_interrupt(Trigger::FallingEdge, pong).unwrap();
    pin3.set_async_interrupt(Trigger::FallingEdge, nothin).unwrap();
    
    loop {}
  
}
