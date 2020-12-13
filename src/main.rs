#![no_std]
#![no_main]

use embedded_hal;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::serial::Write;
use nb;
use riscv;

use numtoa::NumToA;
use arrayvec::ArrayString;

extern crate panic_halt;
use litex_pac as pac;
use litex_hal as hal;
use riscv_rt::entry;
use display_interface_spi::{SPIInterface, SPIInterfaceNoCS};

use embedded_graphics::{
    fonts::{Font6x12, Text},
    prelude::*,
    pixelcolor::Rgb565,
    primitives::{Circle, Rectangle, Triangle},
    style::PrimitiveStyleBuilder,
    style::TextStyleBuilder,
};

use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
use st7789::{ST7789, Orientation};
use display_interface::WriteOnlyDataCommand;
use embedded_graphics::fonts::{Font12x16, Font6x6};

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

/*
fn ssd1331<SPI,DC,CS,RST>(spi: SPI, dc: DC, csn: CS, rstn: RST, delay_source: &mut impl DelayUs<u32>) -> Ssd1331<SPI,DC>
    where SPI: embedded_hal::blocking::spi::Write<u8>,
          DC: OutputPin,
          CS: OutputPin,
          RST: OutputPin,
{
    csn.set_high().unwrap();
    csn.set_low().unwrap();

    let mut display = Ssd1331::new(spi, dc, Rotate0);
    display.reset(&mut rstn, &mut delay_source).unwrap();
    display.init().unwrap();
    display.flush().unwrap();
    display
}
*/

fn st7789<SPI, DC, CS, RST>(spi: SPI, dc: DC, csn: CS, rstn: RST, delay_source: &mut impl DelayUs<u32>) -> ST7789<SPIInterface<SPI, DC, CS>,RST>
    where SPI: embedded_hal::blocking::spi::Write<u8>,
          DC: OutputPin,
          CS: OutputPin,
          RST: OutputPin
{
    // display interface abstraction from SPI and DC
    let di = SPIInterface::new(spi, dc, csn);

    // create driver
    let mut display = ST7789::new(di, rstn, 240, 240);
    display.init(delay_source);//.unwrap();
    display.set_orientation(Orientation::PortraitSwapped);
    delay_source.delay_us(10);
    //display.flush().unwrap();

    display
}


fn st7789_nocs<SPI, DC, RST>(spi: SPI, dc: DC, rstn: RST, delay_source: &mut impl DelayUs<u32>) -> ST7789<SPIInterfaceNoCS<SPI, DC>, RST>
    where SPI: embedded_hal::blocking::spi::Write<u8>,
          DC: OutputPin,
          RST: OutputPin
{
    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(spi, dc);

//    csn.set_high();
//    delay.delay_us(1000u32);
//    csn.set_low();
//    delay.delay_us(1000u32);

    // create driver
    let mut display = ST7789::new(di, rstn, 240, 240);
    display.init(delay_source);//.unwrap();
    display.set_orientation(Orientation::PortraitSwapped);
    delay_source.delay_us(10);
    display
}

// This is the entry point for the application.
// It is not allowed to return.
#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let mut serial = UART {
        registers: peripherals.UART,
    };

    let dc = CTL { index: 0 };
    let mut rstn = CTL { index: 1 };
    let mut csn = CTL { index: 2 };
    let spi = SPI {
        registers: peripherals.OLED_SPI
    };
    let mut delay = TIMER {
        registers: peripherals.TIMER0,
        sys_clk: 50_000_000,
    };

    //let mut display = ssd1331(spi, dc, csn, rstn, &mut delay);
    let mut display = st7789(spi, dc, csn, rstn, &mut delay);
//    let mut display = st7789_nocs(spi, dc, rstn, &mut delay);

    let mut i: u8 = 0;
    let mut num_buffer = [0u8; 20];
    let mut text = ArrayString::<[_; 256]>::new();

    loop {
        i = i.wrapping_add(1);
        text.clear();
        text.push_str("HARRISON\nROCKS\n");
        text.push_str(i.numtoa_str(10, &mut num_buffer));
        text.push_str("\n");
        serial.bwrite_all(text.as_bytes()).unwrap();

        for j in 0..8 {
            if (i & (1u8 << j)) > 0 {
                let _ = LEDS { index: j }.set_high();
            } else {
                let _ = LEDS { index: j }.set_low();
            }
        }

        display.clear(Rgb565::BLACK);

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(Rgb565::YELLOW)
            .build();

        // triangle
        Triangle::new(
            Point::new(16, 16 ),
            Point::new(16 + 16, 16 ),
            Point::new(16 + 8, 0 ),
        )
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        Text::new(&text, Point::new(0, 24 ))
            .into_styled(
                TextStyleBuilder::new(Font6x6)
                    .text_color(Rgb565::BLUE)
                    .build(),
            )
            .draw(&mut display)
            .unwrap();

        //display.flush().unwrap();

        delay.delay_ms(1000 as u32);
        // do some graphics stuff in here
    }
}
