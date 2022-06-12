//! Draw an RGB565 BMP image onto the display by converting the `Rgb565` pixel color type to
//! `BinaryColor` using a simple threshold where any pixel with a value greater than zero is treated
//! as "on".
//!
//! Note that the `bmp` feature for `embedded-graphics` must be turned on.
//!
//! This example is for the `ESP32 DevKit v1` board using I2C0.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> GPIO21
//! (green)  SCL -> GPIO22
//! ```
//!
//! Run on a Blue Pill with `cargo run --example esp32_i2c`.

use std::thread;
use std::time::Duration;
use ssd1327::{prelude::*, I2CDisplayInterface, Ssd1327};
use esp_idf_hal::{self, peripherals::Peripherals, prelude::*};
use tinybmp::Bmp;
use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*};

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
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
   
    let bmp = Bmp::from_slice(include_bytes!("./rust.bmp")).expect("Failed to load BMP image");

    // The image is an RGB565 encoded BMP, so specifying the type as `Image<Bmp<Rgb565>>` will read
    // the pixels correctly
    let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(32, 32));
    
    // We use the `color_converted` method here to automatically convert the RGB565 image data into
    // BinaryColor values.
    im.draw(&mut display.color_converted()).unwrap();
    
    display.flush().unwrap();

    let mut count_down = 100;
    loop {
        if count_down == 0 {
            break;
        }
        println!("CountDown {}", count_down);
        count_down -= 1;
        
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));
    }

    println!("Exiting");

    Ok(())
}
