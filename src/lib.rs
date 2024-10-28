#![no_std]

#[macro_use]
mod dht_common;

use embedded_hal::delay::DelayNs;

mod dht {
    #[cfg(feature = "rp2040")]
    define_dht!(rp2040_hal);

    #[cfg(feature = "rp235x")]
    define_dht!(rp235x_hal);
}

use dht::hal::{
    self,
    gpio::AnyPin,
    pio::{PIOExt, StateMachineIndex, UninitStateMachine},
};
use dht::DhtPio;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DhtError {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch(u32, u32),
    /// FIFO Read error
    ReadError,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DhtResult<T, H> {
    pub temperature: T,
    pub humidity: H,
}

type Dht22Result = DhtResult<f32, f32>;
type Dht11Result = DhtResult<u16, u16>;

pub struct Dht22<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<1, P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht22<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        pio: hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
        clocks: &hal::clocks::ClocksManager,
    ) -> Self {
        Self {
            dht: DhtPio::new(pio, sm, dht_pin, clocks),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn read<D: DelayNs>(&mut self, delay: &mut D) -> Result<Dht22Result, DhtError> {
        let (raw_temp, raw_hum) = self.dht.read_data(delay)?;
        let mut final_t = f32::from(raw_temp & 0x7FFF);

        if (raw_temp & 0x8000) > 0 {
            final_t *= -1.0;
        }

        Ok(DhtResult {
            temperature: final_t / 10.0,
            humidity: f32::from(raw_hum) / 10.0,
        })
    }
}

pub struct Dht22Type2<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<1, P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht22Type2<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        pio: hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
        clocks: &hal::clocks::ClocksManager,
    ) -> Self {
        Self {
            dht: DhtPio::new(pio, sm, dht_pin, clocks),
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::cast_possible_wrap)]
    pub fn read<D: DelayNs>(&mut self, delay: &mut D) -> Result<Dht22Result, DhtError> {
        let (raw_temp, raw_hum) = self.dht.read_data(delay)?;

        let tmp = raw_temp as i16;

        Ok(DhtResult {
            temperature: f32::from(tmp) / 10.0,
            humidity: f32::from(raw_hum) / 10.0,
        })
    }
}

pub struct Dht11<P: PIOExt, STI: StateMachineIndex> {
    dht: DhtPio<18, P, STI>,
}

impl<P: PIOExt, STI: StateMachineIndex> Dht11<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        pio: hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
        clocks: &hal::clocks::ClocksManager,
    ) -> Self {
        Self {
            dht: DhtPio::new(pio, sm, dht_pin, clocks),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn read<D: DelayNs>(&mut self, delay: &mut D) -> Result<Dht11Result, DhtError> {
        let (t, h) = self.dht.read_data(delay)?;
        let mut final_t = (t & 0x7FFF) >> 8;

        if (t & 0x8000) > 0 {
            final_t = 0xFF - final_t;
        }

        Ok(DhtResult {
            temperature: final_t,
            humidity: h >> 8,
        })
    }
}
