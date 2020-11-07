#![no_std]
#![no_main]

use embedded_hal;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::serial::Write;
use nb;
use riscv;

extern crate panic_halt;
use litex_pac as pac;
use litex_hal as hal;
use riscv_rt::entry;

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle, Triangle},
    style::PrimitiveStyleBuilder,
    style::TextStyleBuilder,
};

use ssd1331::{DisplayRotation, Ssd1331};

hal::uart! {
    UART: pac::UART,
}

hal::gpio! {
    CTL: pac::OLED_CTL,
    LEDS: pac::LEDS,
}

hal::spi! {
    SPI: (pac::OLED_SPI, u8),
}

hal::timer! {
    TIMER: pac::TIMER0,
}

// This is the entry point for the application.
// It is not allowed to return.
#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let mut serial = UART {
        registers: peripherals.UART,
    };

    serial.bwrite_all(b"Hello world!\n").unwrap();

    for i in 0..8 {
        if i % 2 == 0 {
            LEDS { index: i }.set_high().unwrap();
        }
    }

    let mut dc = CTL { index: 0 };
    let mut rstn = CTL { index: 1 };
    let mut csn = CTL { index: 2 };
    let mut spi = SPI {
        registers: peripherals.OLED_SPI
    };
    let mut delay = TIMER {
        registers: peripherals.TIMER0,
        sys_clk: 50_000_000,
    };

    csn.set_low().unwrap();
    let interface = display_interface_spi::SPIInterface::new(spi, dc, csn);

    loop {
        // do some graphics stuff in here
    }
}
