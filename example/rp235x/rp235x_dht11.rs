#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;

use defmt_rtt as _;
use panic_halt as _;

use hal::{pac, pio::PIOExt};
use rp235x_hal as hal;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
const DELAY: u32 = 1_000u32;

#[hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut delay = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let (dht_pio, dht_sm, ..) = pac.PIO0.split(&mut pac.RESETS);
    let mut dht = dht_pio::Dht11::new(dht_pio, dht_sm, pins.gpio0.into_function(), &clocks);

    defmt::info!("DHT11 rp235x");

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
    }
}

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_program_name!(c"rp235x-dht11"),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"DHT11 for rp235x"),
    hal::binary_info::rp_program_url!(c"https://github.com/jnthbdn/rs-dht-pio"),
    hal::binary_info::rp_program_build_attribute!(),
];
