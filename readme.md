# DHT PIO Rust Library
[![crates.io](https://img.shields.io/crates/v/dht-pio)](https://crates.io/crates/dht-pio) [![MIT](https://img.shields.io/github/license/jnthbdn/rs-dht-pio)](https://opensource.org/licenses/MIT) [![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/jnthbdn/rs-dht-pio)

_[french version](readme_fr.md)_

## Why?
The DHT (22 or 11) uses a 1-Wire protocol, which is not compatible with the [Dallas Semicondutors](https://en.wikipedia.org/wiki/1-Wire) protocol of the same name. The [Raspberry Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) (like other microcontrollers) has no dedicated peripherals for this protocol. 

Numerous crates exist for using DHT via a digital pin, but after testing several of them, they don't work reliably. The main problem is the implementation of the [embedded_hal](https://crates.io/crates/embedded-hal) by [rp2040_hal](https://crates.io/crates/rp2040-hal). Manipulating the state and direction of a pin takes too long (I measured between 2µs and 6µs depending on the action requested). This is due, among other things, to the impossibility of placing a pin in open drain, which requires "simulating" this feature

## The PIO ❤️
The RP2040 chip (used for the Pico) has a rather atypical peripheral called PIO (Programmable Input/Output), [Chapter 3 of the DataSheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf). In simple terms, the idea is to be able to run a small program (max. 32 instructions), which executes independently. It can manipulate GPIOs and share information with the main program.

The PIO is programmed using an assembler called `pioasm`, with just a few very basic instructions. What's interesting is that each instruction takes (usually) 1 cycle to execute. What's more, it's possible to divide the clock at which the program executes. In our case, we divide the main clock of 125 MHz by 125, giving us one instruction per microsecond.

## Usage
First, create and retrieve the PIO objects
```rust
let (dht_pio, dht_sm, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
```
To create a new object:
- DHT22  
  ````rust
  let mut dht = Dht22::new(dht_pio, dht_sm, pins.gpio0.into_function());
  ```
- DHT11
  ````rust
  let mut dht = Dht11::new(dht_pio, dht_sm, pins.gpio0.into_function());
  ```

Read data:
````rust
let dht_data = dht.read(&mut delay);
```

NB: `read` retrun a `Result<DhtResult, DhtError>`.

## Support
### Board
For the moment, the crates have only been tested on a Raspberry Pico.

### DHT
✅ DHT22  
❔ DHT11

## TODO
- [ ] Finish Readme
- [x] Add CRC read
- [x] Check CRC
- [x] DHT11 support
- [ ] Test DHT11
- [ ] Document code