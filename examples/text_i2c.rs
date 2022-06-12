//! Print "Hello world!" with "Hello rust!" underneath. Uses the `embedded_graphics` crate to draw
//! the text with a 6x8 pixel font.
//!
//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> PB9
//! (green)  SCL -> PB8
//! ```
//!
//! Run on a Blue Pill with `cargo run --example text_i2c`.

use ssd1327::{prelude::*, I2CDisplayInterface, Ssd1327, size::DisplaySize128x128};
use esp_idf_hal::{self, peripherals::Peripherals, prelude::*};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

fn main() -> ! {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    println!("Starting I2C MSP1503 test");

    let config = <esp_idf_hal::i2c::config::MasterConfig as Default>::default().baudrate(400.kHz().into());
    let i2c = esp_idf_hal::i2c::Master::new(
        i2c, 
        esp_idf_hal::i2c::MasterPins { sda, scl }, 
        config
    ).expect("Failed to create I2C master");

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1327::new(interface, DisplaySize128x128, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.clear();
    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();

    // Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // display.flush().unwrap();

    loop {}
}
