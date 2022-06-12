# SSD1306 driver

<!-- [![Build Status](https://circleci.com/gh/jamwaffles/ssd1306/tree/master.svg?style=shield)](https://circleci.com/gh/jamwaffles/ssd1306/tree/master)
[![Crates.io](https://img.shields.io/crates/v/ssd1306.svg)](https://crates.io/crates/ssd1306)
[![Docs.rs](https://docs.rs/ssd1306/badge.svg)](https://docs.rs/ssd1306) -->

[![CRIUS display showing the Rust logo](readme_banner.png?raw=true)](examples/esp32_i2c.rs)

I2C driver for the SSD1327 OLED display.

Please consider [becoming a sponsor](https://github.com/sponsors/jamwaffles/) so I may continue to maintain this crate in my spare time!

## [Documentation](https://docs.rs/ssd1327)

## [Changelog](CHANGELOG.md)

## [Examples](examples)

This crate uses [`probe-run`](https://crates.io/crates/probe-run) to run the examples. Once set up,
it should be as simple as `cargo run --example <example name> --release`.

From [`examples/esp32_i2c.rs`](examples/esp32_i2c.rs):

```rust
use ssd1327::{prelude::*, I2CDisplayInterface, Ssd1327};
use esp_idf_hal::{self, peripherals::Peripherals, prelude::*};
use tinybmp::Bmp;
use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*};

fn main() -> ! {
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

    loop {}
}
```

## Inspiration

[jamwaffles/ssd1306](https://github.com/jamwaffles/ssd1306)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
