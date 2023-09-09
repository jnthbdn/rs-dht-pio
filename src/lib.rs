#![no_std]

use cortex_m::delay::Delay;
use rp2040_hal::gpio::AnyPin;
use rp2040_hal::pio::UninitStateMachine;
use rp2040_hal::pio::{PIOExt, StateMachineIndex};

mod dht;
use dht::DhtPio;

#[derive(Debug)]
pub enum DhtError {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch(u32, u32),
    /// FIFO Read error
    ReadError,
}

#[derive(Debug)]
pub struct DhtResult {
    pub temperature: f32,
    pub humidity: f32,
}

pub struct Dht22<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht22<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        pio: rp2040_hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
    ) -> Self {
        Self {
            dht: DhtPio::new(pio, sm, dht_pin),
        }
    }

    pub fn read(&mut self, delay: &mut Delay) -> Result<DhtResult, DhtError> {
        let (t, h) = self.dht.read_data(delay)?;
        let mut final_t: i32 = (t & 0x7FFF) as i32;

        if (t & 0x8000) > 0 {
            final_t *= -1;
        }

        Ok(DhtResult {
            temperature: final_t as f32 / 10.0,
            humidity: h as f32 / 10.0,
        })
    }
}
pub struct Dht11<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht11<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        pio: rp2040_hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
    ) -> Self {
        Self {
            dht: DhtPio::new(pio, sm, dht_pin),
        }
    }

    pub fn read(&mut self, delay: &mut Delay) -> Result<DhtResult, DhtError> {
        let (t, h) = self.dht.read_data(delay)?;
        let mut final_t: i32 = ((t & 0x7FFF) >> 8) as i32;

        if (t & 0x8000) > 0 {
            final_t *= -1;
        }

        Ok(DhtResult {
            temperature: final_t as f32,
            humidity: (h >> 8) as f32,
        })
    }
}
