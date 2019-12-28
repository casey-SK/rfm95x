# RFM9x 

A Rust driver for the RFM9x LoRa/FSK transceiver module (work in progress).

Author: Tommy van der Vorst (Pixelspark)

## What works

Transmission of LoRA frames was successfully tested against The Things Network (needs LoRaWAN logic). Receiving currently
does not work.

Currently uses the `rppal` library to command the Raspbery Pi SPI bus, so will not work (yet) on other systems.

## Usage

See [the basic example](./examples/basic.rs).

## Copyright & license

Copyright (C) 2019-2020 Tommy van der Vorst, Pixelspark. Released under the  [MIT license](./LICENSE).

