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
        let (raw_temp, raw_hum) = self.dht.read_data(delay)?;
        let mut final_t: f32 = (raw_temp & 0x7FFF) as f32;

        if (raw_temp & 0x8000) > 0 {
            final_t *= -1.0;
        }

        Ok(DhtResult {
            temperature: final_t / 10.0,
            humidity: raw_hum as f32 / 10.0,
        })
    }
}

pub struct Dht22Type2<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht22Type2<P, STI> {
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
        let (raw_temp, raw_hum) = self.dht.read_data(delay)?;

        let tmp: i16 = raw_temp as i16;

        Ok(DhtResult {
            temperature: (tmp as f32) / 10.0,
            humidity: raw_hum as f32 / 10.0,
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
        let mut final_t: f32 = ((t & 0x7FFF) >> 8) as f32;

        if (t & 0x8000) > 0 {
            final_t *= -1.0;
        }

        Ok(DhtResult {
            temperature: final_t,
            humidity: (h >> 8) as f32,
        })
    }
}
