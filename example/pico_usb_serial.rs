use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use cortex_m::interrupt::free;
use rp_pico::hal::pac;
use rp_pico::hal::pac::interrupt;
use rp_pico::hal::usb::UsbBus;

use usb_device::{
    class_prelude::UsbBusAllocator,
    prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;

use crate::serial_buffer::SerialBuffer;

// use crate::{circular_buffer::CircularBuffer, simple_buffer::SimpleBuffer};

static WRITE_BUFFER_SIZE: usize = 32;
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
static mut USB_BUFFER: Option<Box<dyn SerialBuffer>> = None;

pub struct PicoUsbSerial {}

impl PicoUsbSerial {
    pub fn init(
        bus: UsbBus,
        buffer: Box<dyn SerialBuffer>,
        manufacturer: &'static str,
        product: &'static str,
        serial_number: &'static str,
    ) -> Result<(), String> {
        unsafe {
            if USB_BUS.is_some() {
                return Err("PicoUsbSerial already initialized.".to_string());
            }

            // Safety: This is safe as interrupts haven't been started yet
            USB_BUFFER = Some(buffer);
            USB_BUS = Some(UsbBusAllocator::new(bus));
        }

        let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

        let serial = SerialPort::new(bus_ref);
        let device = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x2E8A, 0x0005))
            .manufacturer(manufacturer)
            .product(product)
            .serial_number(serial_number)
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();

        unsafe {
            USB_SERIAL = Some(serial);
            USB_DEVICE = Some(device);

            // Enable the USB interrupt
            pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        };

        Ok(())
    }

    pub fn get_serial() -> Result<Self, String> {
        unsafe {
            if USB_BUS.is_none() {
                Err("Serial not initialized.".to_string())
            } else {
                Ok(Self {})
            }
        }
    }

    pub fn available_data(&self) -> usize {
        unsafe {
            return USB_BUFFER.as_ref().unwrap().available_to_read();
        };
    }

    pub fn read(&self) -> u8 {
        free(|_cs| unsafe { USB_BUFFER.as_mut().unwrap().read().unwrap_or(0) })
    }

    pub fn write(&self, data: &[u8]) -> usb_device::Result<usize> {
        free(|_cs| {
            let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

            // let mut wr = data;
            let data_chunks = data.chunks(WRITE_BUFFER_SIZE);

            for mut buf in data_chunks {
                while !buf.is_empty() {
                    serial.write(buf).map(|len| {
                        buf = &buf[len..];
                    })?;
                }
            }

            Ok(data.len())
        })
    }
}

/// This function is called whenever the USB Hardware generates an Interrupt
/// Request.
///
/// We do all our USB work under interrupt, so the main thread can continue on
/// knowing nothing about USB.
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    let buffer = USB_BUFFER.as_mut().unwrap();

    // Grab the global objects. This is OK as we only access them under interrupt.
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let serial = USB_SERIAL.as_mut().unwrap();

    // Poll the USB driver with all of our supported USB Classes
    if usb_dev.poll(&mut [serial]) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_e) => {
                // Do nothing
            }
            Ok(0) => {
                // Do nothing
            }
            Ok(count) => {
                // Convert to upper case
                buf.iter_mut().take(count).for_each(|b| {
                    buffer.write(b.clone());
                });
            }
        }
    }
}
