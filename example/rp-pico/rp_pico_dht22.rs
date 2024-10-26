#![no_std]
#![no_main]

extern crate alloc;
extern crate dht_pio;

use defmt_rtt as _;

use embedded_hal::delay::DelayNs;

use alloc::boxed::Box;
use alloc::format;
use core::panic::PanicInfo;
use cortex_m::interrupt::free;
use dht_pio::Dht22;
use embedded_alloc::LlffHeap as Heap;

use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pio::PIOExt;

use hal::{clocks::init_clocks_and_plls, pac, usb::UsbBus, Sio, Watchdog};

mod pico_usb_serial;
mod serial_buffer;
mod simple_buffer;
use pico_usb_serial::PicoUsbSerial;
use serial_buffer::SerialBuffer;
use simple_buffer::SimpleBuffer;

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 4069;

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    init_heap();

    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    PicoUsbSerial::init(
        UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        ),
        Box::new(SimpleBuffer::new(1024)),
        "MyCorp",
        "Test",
        "Serial",
    )
    .expect("Failed to init Serial");

    let serial = PicoUsbSerial::get_serial().expect("Failed to get serial!");

    let (dht_pio, dht_sm, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut dht = Dht22::new(dht_pio, dht_sm, pins.gpio0.into_function());

    defmt::info!("DHT22 rp-pico");

    defmt::info!("waiting sensor...");
    delay.delay_ms(1000);
    defmt::info!("...done");

    loop {
        free(|_cs| match dht.read(&mut delay) {
            Ok(result) => {
                defmt::info!("DHT22 {:#?}", result);
                serial.write(format!("{:#?}\r\n", result).as_bytes()).ok();
            }
            Err(e) => {
                defmt::error!("DHT22 {}", e);
                serial
                    .write(format!("DHT Error: {:?}\r\n", e).as_bytes())
                    .ok();
            }
        });

        delay.delay_ms(1000);
    }
}

fn init_heap() {
    use core::mem::MaybeUninit;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    let ser = PicoUsbSerial::get_serial();

    if ser.is_ok() {
        let ser = ser.unwrap();
        let _ = ser.write(b"===== PANIC =====\r\n");
    }

    loop {}
}
