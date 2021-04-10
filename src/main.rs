#![no_std]
#![no_main]

use embedded_hal;
use embedded_hal::digital::v2::{OutputPin};
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
    fonts::*,
    prelude::*,
    pixelcolor::{Rgb565, raw::RawU16},
    primitives::{Circle, Rectangle, Triangle},
    style::PrimitiveStyleBuilder,
    style::TextStyleBuilder,
};

use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
use st7789::{ST7789, Orientation};
use display_interface::WriteOnlyDataCommand;

use oorandom::*;

hal::uart! {
    UART: pac::UART,
}

hal::gpio! {
    CTL: pac::OLED_CTL,
    LEDS: pac::LEDS,

    // something wrong with this
    //MATRIX: pac::MATRIX_SPI,
}

hal::spi! {
    SPI: (pac::OLED_SPI, u8),
}

hal::timer! {
    TIMER: pac::TIMER0,
}


fn ssd1331<SPI, DC, CS, RST, PinE>(spi: SPI, dc: DC, csn: &mut CS, rstn: &mut RST, delay_source: &mut impl DelayMs<u8>) -> Ssd1331<SPI, DC>
    where SPI: embedded_hal::blocking::spi::Write<u8>,
          DC: OutputPin<Error = PinE>,
          CS: OutputPin<Error = PinE>,
          RST: OutputPin<Error = PinE>,
{
    csn.set_high();
    csn.set_low();

    let mut display = Ssd1331::new(spi, dc, Rotate0);
    display.reset(rstn, delay_source);
    display.init();
    display.flush();
    display
}


fn st7789<SPI, DC, CS, RST>(spi: SPI, dc: DC, csn: CS, rstn: RST, delay_source: &mut impl DelayUs<u32>) -> ST7789<SPIInterface<SPI, DC, CS>, RST>
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
    let matrix_raw = 0xC000_0000 as *mut u32;
    let matrix = unsafe { core::slice::from_raw_parts_mut(matrix_raw, 8) };

    matrix[0] = 0x05500550;
    matrix[1] = 0x54455445;
    matrix[2] = 0x54444445;
    matrix[3] = 0x54444445;
    matrix[4] = 0x54444445;
    matrix[5] = 0x05444450;
    matrix[6] = 0x00544500;
    matrix[7] = 0x00055000;

    // TODO make this work
    /*
    let m0 = MATRIX { index: 0 };
    let m1 = MATRIX { index: 1 };
*/
    let mut delay_source = TIMER {
        registers: peripherals.TIMER0,
        sys_clk: 25_000_000,
    };
    csn.set_high();
    csn.set_low();

    let mut display = ssd1331(spi, dc, &mut csn, &mut rstn, &mut delay_source);
//    let mut display = st7789(spi, dc, csn, rstn, &mut delay_source);
//   let mut display = st7789_nocs(spi, dc, rstn, &mut delay_source);

    let mut i: u32 = 0;
    let mut num_buffer = [0u8; 20];
    let mut text = ArrayString::<[_; 1024]>::new();

    let mut rng = oorandom::Rand32::new(matrix[0].into());

    loop {
        i = i.wrapping_add(1);
        for j in 0..8 {
            if ((i as u8) & (1u8 << j)) > 0 {
                let _ = LEDS { index: j }.set_high();
            } else {
                let _ = LEDS { index: j }.set_low();
            }
        }


        display.clear();
        text.clear();
        for i in 0..8 {
            // TODO write a left-padding number formatter
            text.push_str(i.numtoa_str(10, &mut num_buffer));
            text.push_str(" 0x");
            text.push_str((matrix[i]).numtoa_str(16, &mut num_buffer));
            text.push_str("\n");
        }
//        serial.bwrite_all(text.as_bytes()).unwrap();

        Text::new(&text, Point::new(0, 0 ))
            .into_styled(
                TextStyleBuilder::new(Font6x8)
                    .text_color(Rgb565::BLUE)
                    .build(),
            )
            .draw(&mut display)
            .unwrap();

        // this flushes the ssd1331 framebuffer entirely to the ssd1331.
        display.flush();

        delay_source.delay_ms(100 as u32);

        for i in 0..8 {
            let x = matrix[i];
            let mut y = 0;

            for j in 0..8 {
                let n = (x >> 4*j) & 0x07;
                let m = match n {
                    0 => 0,
                    1 => 7,
                    n => n-1
                };
                y |= (m << 4*j);
            }
            matrix[i] = y;
        }

        // something wrong with this, it's speed not out so maybe the macro doesn't work?
        // maybe just do it with unsafe directly?
        /*
        if i % 3 == 0 {
            m0.set_high();
        } else {
            m0.set_low();
        }
        if i % 7 == 0 {
            m1.set_high();
        } else {
            m1.set_low();
        }
*/

/*
        // now draw our ad hoc hw accelerated things
        let raw_yellow = RawU16::from(Rgb565::YELLOW).into_inner();

        display.draw_hw_line(16, 16, 32, 16, raw_yellow);
        display.draw_hw_line(32, 16, 24, 0, raw_yellow);
        display.draw_hw_line(24, 0, 16, 16, raw_yellow);

        //display.draw_hw_rect(16, 16, 32, 16, raw_yellow, None);
        //display.draw_hw_rect(32, 16, 24, 0, raw_yellow, None);
        //display.draw_hw_rect(24, 0, 16, 16, raw_yellow, None);

        let raw_green = RawU16::from(Rgb565::GREEN).into_inner();
        let raw_red = RawU16::from(Rgb565::RED).into_inner();
        let raw_cyan = RawU16::from(Rgb565::CYAN).into_inner();
        display.draw_hw_rect(34, 0, 95, 63, raw_red, Some(raw_green), &mut delay_source);
*/

        // do some graphics stuff in here
    }
}
