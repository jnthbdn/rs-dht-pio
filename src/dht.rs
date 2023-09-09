use cortex_m::delay::Delay;
use pio_proc::pio_file;
use rp2040_hal::gpio::AnyPin;
use rp2040_hal::pio::{PIOExt, Running, StateMachine, StateMachineIndex, Tx};
use rp2040_hal::pio::{Rx, UninitStateMachine};

use crate::DhtError;

pub struct DhtPio<P: PIOExt, STI: StateMachineIndex> {
    // sm: PicoStateMachine<P, STI>,
    sm: StateMachine<(P, STI), Running>,
    rx_fifo: Rx<(P, STI)>,
    tx_fifo: Tx<(P, STI)>,
}

impl<P: PIOExt, STI: StateMachineIndex> DhtPio<P, STI> {
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
            .push_threshold(32)
            .build(sm);
        sm.set_pindirs([(pin.id().num, rp2040_hal::pio::PinDir::Output)]);

        Self {
            sm: sm.start(),
            rx_fifo: rx,
            tx_fifo: tx,
        }
    }

    pub fn read_data(&mut self, delay: &mut Delay) -> Result<(u32, u32), DhtError> {
        let mut timeout = 2000;
        let mut raw: [Option<u32>; 2] = [None; 2];

        self.tx_fifo.write(1);

        while timeout > 0 && raw[1].is_none() {
            match self.rx_fifo.read() {
                Some(d) => {
                    if raw[0].is_none() {
                        raw[0] = Some(d);
                    } else {
                        raw[1] = Some(d);
                    }
                }
                None => (),
            }

            delay.delay_ms(1);
            timeout -= 1;
        }

        if timeout <= 0 {
            self.sm.restart();
            return Err(DhtError::Timeout);
        }

        let data = raw[0].unwrap();
        let crc = raw[1].unwrap();

        if Self::compute_crc(data) != crc {
            return Err(DhtError::CrcMismatch(
                raw[0].unwrap_or(0),
                raw[1].unwrap_or(0),
            ));
        }

        Ok((data & 0x0000FFFF, (data & 0xFFFF0000) >> 16))
    }

    fn compute_crc(data: u32) -> u32 {
        let mut crc: u32 = 0;
        crc += data & 0x000000FF;
        crc += (data & 0x0000FF00) >> 8;
        crc += (data & 0x00FF0000) >> 16;
        crc += (data & 0xFF000000) >> 24;

        crc % 256
    }
}
