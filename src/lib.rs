#![no_std]

use cortex_m::delay::Delay;
use pio_proc::pio_file;
use rp2040_hal::gpio::AnyPin;
use rp2040_hal::pio::{PIOExt, Running, StateMachine, StateMachineIndex, Tx};
use rp2040_hal::pio::{Rx, UninitStateMachine};

#[derive(Debug)]
pub enum DhtError {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch,
    /// FIFO Read error
    ReadError,
}

#[derive(Debug)]
pub struct DhtResult {
    pub temperature: f32,
    pub humidity: f32,
}

pub struct PicoDht<P: PIOExt, STI: StateMachineIndex> {
    // sm: PicoStateMachine<P, STI>,
    sm: StateMachine<(P, STI), Running>,
    rx_fifo: Rx<(P, STI)>,
    tx_fifo: Tx<(P, STI)>,
}

impl<P: PIOExt, STI: StateMachineIndex> PicoDht<P, STI> {
    pub fn new<I: AnyPin<Function = P::PinFunction>>(
        mut pio: rp2040_hal::pio::PIO<P>,
        sm: UninitStateMachine<(P, STI)>,
        dht_pin: I,
    ) -> Self {
        let program = pio_file!("./src/dht.pio");

        let pin = dht_pin.into();

        let installed = pio.install(&program.program).unwrap();

        let (int, frac) = (125, 0);
        let (mut sm, rx, tx) = rp2040_hal::pio::PIOBuilder::from_program(installed)
            .set_pins(pin.id().num, 1)
            .clock_divisor_fixed_point(int, frac)
            .build(sm);
        sm.set_pindirs([(pin.id().num, rp2040_hal::pio::PinDir::Output)]);

        Self {
            sm: sm.start(),
            rx_fifo: rx,
            tx_fifo: tx,
        }
    }

    /// Read data from the sensor. This blocking function (for maximum timeout of 2 seconds).
    pub fn read_data(&mut self, delay: &mut Delay) -> Result<DhtResult, DhtError> {
        let mut timeout = 2000;

        self.tx_fifo.write(1);

        while timeout > 0 && self.rx_fifo.is_empty() {
            delay.delay_ms(1);
            timeout -= 1;
        }

        if timeout <= 0 {
            self.sm.restart();
            return Err(DhtError::Timeout);
        }

        let raw = match self.rx_fifo.read() {
            Some(d) => d,
            None => {
                self.sm.restart();
                return Err(DhtError::ReadError);
            }
        };

        let t_raw = raw & 0x0000FFFF;
        let h_raw = (raw & 0xFFFF0000) >> 16;

        Ok(DhtResult {
            temperature: t_raw as f32 / 10.0,
            humidity: h_raw as f32 / 10.0,
        })
    }
}
