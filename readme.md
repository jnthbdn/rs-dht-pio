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
```rust
let dht_data = dht.read(&mut delay);
```

NB: `read` return a `Result<DhtResult, DhtError>`.

### DHT22 type 2 🧐
It seems that there are two versions of DHT22. I haven't found anything really conclusive, but what is certain is that not all DHT22s have the same data format... In one case the format is the same as presented in (almost) all datasheets, i.e. the most significant bit is set to `1` if the number is negative, **but** the binary representation of the absolute temperature value is not changed. For example: 
  - `0000 0000 0110 1001` = 105 or 10.5°C
  - `1000 0000 0110 1001` = 32873 or -10.5°C

This is how the `Dht22` struct will "decode" the data coming from the sensor.
However, I've come across sensors that don't work like this at all. But in a (ultimately) more logical way. Since the data is represented in [**two's complement**](https://en.wikipedia.org/wiki/Two%27s_complement). In this case, use `Dht22Type2`. For example: 
  - `0000 0000 0110 1001` = 105 i.e. 10.5°C
  - `1111 1111 1001 0111` = 65431 i.e. -10.5°C

To simplify, if your sensor is a DHT22 but the values don't seem consistent (negative values), then try "Type 2" (and if nothing really works, open an exit 😉 ).


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

## Thanks
 <img src="https://avatars.githubusercontent.com/u/10778792?v=4" style="width: 40px; border-radius: 50%; vertical-align: middle;" /> [Geir Ertzaas (grukx)](https://github.com/grukx), for actively discover (too many?) bugs.