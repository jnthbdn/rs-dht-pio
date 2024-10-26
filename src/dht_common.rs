macro_rules! define_dht {
    ($hal:path, $clock_int:expr) => {
        use embedded_hal::delay::DelayNs;

        use pio_proc::pio_file;

        pub(crate) use $hal as hal;

        use hal::gpio::AnyPin;
        use hal::pio::{PIOExt, Running, StateMachine, StateMachineIndex, Tx};
        use hal::pio::{Rx, ShiftDirection, UninitStateMachine};

        use crate::DhtError;

        pub struct DhtPio<const ID: u32, P: PIOExt, STI: StateMachineIndex> {
            sm: StateMachine<(P, STI), Running>,
            rx_fifo: Rx<(P, STI)>,
            tx_fifo: Tx<(P, STI)>,
        }

        impl<const ID: u32, P: PIOExt, STI: StateMachineIndex> DhtPio<ID, P, STI> {
            pub fn new<I: AnyPin<Function = P::PinFunction>>(
                mut pio: hal::pio::PIO<P>,
                sm: UninitStateMachine<(P, STI)>,
                dht_pin: I,
            ) -> Self {
                let program = pio_file!("./src/dht.pio");

                let pin = dht_pin.into();

                let installed = pio.install(&program.program).unwrap();

                let (int, frac) = ($clock_int, 0);
                let (mut sm, rx, tx) = hal::pio::PIOBuilder::from_installed_program(installed)
                    .out_pins(pin.id().num, 1)
                    .set_pins(pin.id().num, 1)
                    .in_pin_base(pin.id().num)
                    .clock_divisor_fixed_point(int, frac)
                    .push_threshold(32)
                    .out_shift_direction(ShiftDirection::Left)
                    .in_shift_direction(ShiftDirection::Left)
                    .build(sm);
                sm.set_pindirs([(pin.id().num, hal::pio::PinDir::Output)]);

                Self {
                    sm: sm.start(),
                    rx_fifo: rx,
                    tx_fifo: tx,
                }
            }

            pub fn read_data<D: DelayNs>(&mut self, delay: &mut D) -> Result<(u16, u16), DhtError> {
                let mut timeout = 2000;
                let mut raw: [Option<u32>; 2] = [None; 2];

                self.tx_fifo.write(ID);

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

                Ok((
                    (data & 0x0000FFFF) as u16,
                    ((data & 0xFFFF0000) >> 16) as u16,
                ))
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
    };
}
