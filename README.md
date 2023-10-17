# RFM9x rust driver crate for linux systems

Rust driver for the RFM9x LoRa/FSK transceiver module. An example of compatible 
hardware is the Adafruit LoRa Radio Bonnet shown below attached to a 
Raspberry Pi Zero W.

<img src="https://cdn.shopify.com/s/files/1/1004/5324/products/4074-05_1024x1024.jpg?v=1571439709" width="400" />

## Status

Highly experimental, Not recommended for production. Packets can be sent and
received using this crate, but it is not optimized and many bugs still exist.

## Usage

Remember to run `rustup target add arm-unknown-linux-gnueabihf`

The examples folder provides barebones usage of crate functions:
- example: [send](./examples/rpi_tx.rs)
- example: [receive](./examples/rpi_rx.rs)
- example: [async tx/rx](./examples/rpi_async_tx_rx.rs) (work-in-progress)

To build the examples in this repo, you can use `cargo build --example <example_name>`
if you are running the build on a device similar to the one you will be deploying it on.

If you are for example building the binaries on an x86 system and copying them to a 
Raspberry Pi, you will need to use the [cargo cross crate](https://crates.io/crates/cross)
and and build using the following command:

```
cross build --examples --release --target=arm-unknown-linux-gnueabihf
```

You can then copy the file to your raspberry pi using `scp`:

```
scp target/arm-unknown-linux-gnueabihf/release/examples/rpi_tx username@zero1.local:~/rpi_tx

scp target/arm-unknown-linux-gnueabihf/release/examples/rpi_rx username@zero2.local:~/rpi_rx
```

## Copyright & license

Copyright (C) 2019-2020 Tommy van der Vorst, Pixelspark. Released under the  [MIT license](./LICENSE).

Copyright (C) 2023 Casey McMahon

