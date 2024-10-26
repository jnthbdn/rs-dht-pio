#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};

use defmt_rtt as _;
use panic_halt as _;

use hal::{pac, pio::PIOExt};
use rp_pico::hal;

const DELAY: u32 = 2_000u32;
const _: () = assert!(DELAY > 1_000u32, "DELAY must greater or equal 1s");

#[hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut delay = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let (dht_pio, dht_sm, ..) = pac.PIO0.split(&mut pac.RESETS);
    let mut dht = dht_pio::Dht11::new(dht_pio, dht_sm, pins.gpio0.into_function());

    let mut led_pin = pins.gpio25.into_push_pull_output();

    defmt::info!("DHT11 rp-pico");

    led_pin.set_high().unwrap();

    defmt::info!("waiting sensor...");
    delay.delay_ms(DELAY);
    defmt::info!("...done");

    loop {
        match dht.read(&mut delay) {
            Ok(result) => {
                defmt::info!(
                    "DHT11 temperature: {} humidity: {}",
                    result.temperature,
                    result.humidity
                );
            }
            Err(err) => {
                defmt::error!("DHT11 error: {}", err);
            }
        }

        delay.delay_ms(DELAY);

        led_pin.toggle().unwrap();
    }
}
