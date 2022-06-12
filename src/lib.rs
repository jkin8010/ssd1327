//! SSD1327 OLED display driver.
//!
//! This crate provides a driver interface to the SSD1327 monochrome OLED display driver. It
//! supports I2C and SPI via the [`display_interface`](https://docs.rs/display_interface) crate.
//!
//! The main driver is created using [`Ssd1327::new`] which accepts an interface instance, display
//! size, rotation and mode. The following display modes are supported:
//!
//! - [`BasicMode`](crate::mode::BasicMode) - A simple mode with lower level methods available.
//! - [`BufferedGraphicsMode`] - A framebuffered mode with additional methods and integration with
//!   [embedded-graphics](https://docs.rs/embedded-graphics).
//! - [`TerminalMode`] - A bufferless mode supporting drawing text to the display, as well as
//!   setting cursor positions like a simple terminal.
//!
//! # Examples
//!
//! Examples can be found in [the examples/
//! folder](https://github.com/jkin8010/ssd1327/blob/master/examples)
//!
//! ## Draw some text to the display
//!
//! Uses [`BufferedGraphicsMode`] and [embedded_graphics](https://docs.rs/embedded-graphics). [See
//! the complete example
//! here](https://github.com/jkin8010/ssd1327/blob/master/examples/text_i2c.rs).
//!
//! ```rust
//! # use ssd1327::test_helpers::I2cStub;
//! # let i2c = I2cStub;
//! use embedded_graphics::{
//!     mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
//!     pixelcolor::BinaryColor,
//!     prelude::*,
//!     text::{Baseline, Text},
//! };
//! use ssd1327::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1327};
//!
//! let interface = I2CDisplayInterface::new(i2c);
//! let mut display = Ssd1327::new(
//!     interface,
//!     DisplaySize128x64,
//!     DisplayRotation::Rotate0,
//! ).into_buffered_graphics_mode();
//! display.init().unwrap();
//!
//! let text_style = MonoTextStyleBuilder::new()
//!     .font(&FONT_6X10)
//!     .text_color(BinaryColor::On)
//!     .build();
//!
//! Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
//!     .draw(&mut display)
//!     .unwrap();
//!
//! Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
//!     .draw(&mut display)
//!     .unwrap();
//!
//! display.flush().unwrap();
//! ```
//!
//! ## Write text to the display without a framebuffer
//!
//! Uses [`TerminalMode`]. [See the complete example
//! here](https://github.com/jkin8010/ssd1327/blob/master/examples/terminal_i2c.rs).
//!
//! ```rust
//! # use ssd1327::test_helpers::I2cStub;
//! # let i2c = I2cStub;
//! use core::fmt::Write;
//! use ssd1327::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1327};
//!
//! let interface = I2CDisplayInterface::new(i2c);
//!
//! let mut display = Ssd1327::new(
//!     interface,
//!     DisplaySize128x64,
//!     DisplayRotation::Rotate0,
//! ).into_terminal_mode();
//! display.init().unwrap();
//! display.clear().unwrap();
//!
//! // Spam some characters to the display
//! for c in 97..123 {
//!     let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
//! }
//! for c in 65..91 {
//!     let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
//! }
//!
//! // The `write!()` macro is also supported
//! write!(display, "Hello, {}", "world");
//! ```
//!
//! [featureset]: https://github.com/jamwaffles/embedded-graphics#features
//! [`BufferedGraphicsMode`]: crate::mode::BufferedGraphicsMode
//! [`TerminalMode`]: crate::mode::TerminalMode

// #![no_std]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(broken_intra_doc_links)]

mod brightness;
pub mod command;
mod error;
mod i2c_interface;
pub mod mode;
pub mod prelude;
pub mod rotation;
pub mod size;

pub use crate::i2c_interface::I2CDisplayInterface;
use crate::mode::BasicMode;
use brightness::Brightness;
use command::{
    Command, 
    DisplayMode, 
    FrontClockDivideRatio, 
    VcomhLevel, 
    PhaseLength, 
    PreChargeVoltageLevel, 
    NFrames, 
    HScrollDir, 
    generate_remap
};
use display_interface::{DataFormat::U8, DisplayError, WriteOnlyDataCommand};
use display_interface_spi::{SPIInterface, SPIInterfaceNoCS};
use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin};
use error::Error;
use mode::{BufferedGraphicsMode, TerminalMode};
use rotation::DisplayRotation;
use size::DisplaySize;

/// SSD1327 driver.
///
/// Note that some methods are only available when the display is configured in a certain [`mode`].
#[derive(Copy, Clone, Debug)]
pub struct Ssd1327<DI, SIZE, MODE> {
    interface: DI,
    mode: MODE,
    size: SIZE,
    rotation: DisplayRotation,
}

impl<DI, SIZE> Ssd1327<DI, SIZE, BasicMode>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    /// Create a basic SSD1327 interface.
    ///
    /// Use the `into_*_mode` methods to enable more functionality.
    pub fn new(interface: DI, size: SIZE, rotation: DisplayRotation) -> Self {
        Self {
            interface,
            size,
            mode: BasicMode,
            rotation,
        }
    }
}

impl<DI, SIZE, MODE> Ssd1327<DI, SIZE, MODE>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    /// Convert the display into another interface mode.
    fn into_mode<MODE2>(self, mode: MODE2) -> Ssd1327<DI, SIZE, MODE2> {
        Ssd1327 {
            mode,
            interface: self.interface,
            size: self.size,
            rotation: self.rotation,
        }
    }

    /// Convert the display into a buffered graphics mode, supporting
    /// [embedded-graphics](https://crates.io/crates/embedded-graphics).
    ///
    /// See [BufferedGraphicsMode] for more information.
    pub fn into_buffered_graphics_mode(self) -> Ssd1327<DI, SIZE, BufferedGraphicsMode<SIZE>> {
        let mode = BufferedGraphicsMode::new();
        self.into_mode(mode)
    }

    /// Convert the display into a text-only, terminal-like mode.
    ///
    /// See [TerminalMode] for more information.
    pub fn into_terminal_mode(self) -> Ssd1327<DI, SIZE, TerminalMode> {
        self.into_mode(TerminalMode::new())
    }

    /// Initialise the display in one of the available addressing modes.
    pub fn init_basic(&mut self) -> Result<(), DisplayError> {
        let rotation = self.rotation;
        println!("libssd1327: Initialising display with rotation {:?}", rotation);
        // Initialise the display
        Command::DisplayOn(false).send(&mut self.interface)?;
        
        self.size.configure(&mut self.interface)?;
        self.set_rotation(rotation)?;

        Command::DisplayOffset(0x00).send(&mut self.interface)?;
        Command::DisplayMode(DisplayMode::Normal).send(&mut self.interface)?;

        Command::PhaseLength(PhaseLength::PhaseAuto).send(&mut self.interface)?;
        Command::FrontClockDivideRatio(FrontClockDivideRatio::Hz100).send(&mut self.interface)?;
        Command::VoltageRegulator(true).send(&mut self.interface)?;

        self.set_brightness(Brightness::default())?;
   
        Command::VcomhVoltage(VcomhLevel::V082).send(&mut self.interface)?;
        Command::PreChargeVoltage(PreChargeVoltageLevel::V050).send(&mut self.interface)?;
        Command::FunctionSelectionB(0x62).send(&mut self.interface)?;
        Command::CommandLock(false).send(&mut self.interface)?;
        Command::HorizontalScrollSetup(HScrollDir::LeftToRight, 0, 0x3F, 0, 0x7F, NFrames::F2).send(&mut self.interface)?;
        Command::EnableScroll(false).send(&mut self.interface)?;
        
        Command::DisplayOn(true).send(&mut self.interface)?;

        Ok(())
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Cf. `set_draw_area` to modify the affected area by
    /// this method.
    ///
    /// This method takes advantage of a bounding box for faster writes.
    pub fn bounded_draw(
        &mut self,
        buffer: &[u8],
        disp_width: usize,
        upper_left: (u8, u8),
        lower_right: (u8, u8),
    ) -> Result<(), DisplayError> {
        Self::flush_buffer_chunks(
            &mut self.interface,
            buffer,
            disp_width,
            upper_left,
            lower_right,
        )
    }

    /// Send a raw buffer to the display.
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.send_data(U8(&buffer))
    }

    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// ```rust
    /// # use ssd1327::test_helpers::StubInterface;
    /// # let interface = StubInterface;
    /// use ssd1327::{mode::TerminalMode, prelude::*, Ssd1327};
    ///
    /// let mut display = Ssd1327::new(
    ///     interface,
    ///     DisplaySize128x128,
    ///     DisplayRotation::Rotate0,
    /// ).into_terminal_mode();
    /// assert_eq!(display.dimensions(), (128, 128));
    ///
    /// # let interface = StubInterface;
    /// let mut rotated_display = Ssd1327::new(
    ///     interface,
    ///     DisplaySize128x64,
    ///     DisplayRotation::Rotate90,
    /// ).into_terminal_mode();
    /// assert_eq!(rotated_display.dimensions(), (64, 128));
    /// ```
    pub fn dimensions(&self) -> (u8, u8) {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (SIZE::WIDTH, SIZE::HEIGHT),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (SIZE::HEIGHT, SIZE::WIDTH),
        }
    }

    /// Get the display rotation.
    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    /// Set the display rotation.
    pub fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.rotation = rotation;

        match rotation {
            DisplayRotation::Rotate0 => {
                let _remap_flags = generate_remap(
                    false,
                    true,
                    false,
                    true,
                    true,
                    true,
                    false
                );
                Command::Remap(0b0101_1100).send(&mut self.interface)?;
                Command::DisplayStartLine(0x00).send(&mut self.interface)?;
            }
            DisplayRotation::Rotate90 => {
                Command::Remap(0b0101_1000).send(&mut self.interface)?;
                Command::DisplayStartLine(0x00).send(&mut self.interface)?;
            }
            DisplayRotation::Rotate180 => {
                Command::Remap(0b0001_0101).send(&mut self.interface)?;
                Command::DisplayStartLine(0x00).send(&mut self.interface)?;
            }
            DisplayRotation::Rotate270 => {
                Command::Remap(0b0001_0111).send(&mut self.interface)?;
                Command::DisplayStartLine(0x78).send(&mut self.interface)?;
            }
        };

        Ok(())
    }

    /// Set mirror enabled/disabled.
    pub fn set_mirror(&mut self, mirror: bool) -> Result<(), DisplayError> {
        if mirror {
            match self.rotation {
                DisplayRotation::Rotate0 => {
                    Command::Remap(0b0101_1110).send(&mut self.interface)?;
                    Command::DisplayStartLine(0x00).send(&mut self.interface)?;
                }
                DisplayRotation::Rotate90 => {
                    Command::Remap(0b0101_1000).send(&mut self.interface)?;
                    Command::DisplayStartLine(0x00).send(&mut self.interface)?;
                }
                DisplayRotation::Rotate180 => {
                    Command::Remap(0b0001_0101).send(&mut self.interface)?;
                    Command::DisplayStartLine(0x00).send(&mut self.interface)?;
                }
                DisplayRotation::Rotate270 => {
                    Command::Remap(0b0001_0111).send(&mut self.interface)?;
                    Command::DisplayStartLine(0x78).send(&mut self.interface)?;
                }
            };
        } else {
            self.set_rotation(self.rotation)?;
        }
        Ok(())
    }

    /// Change the display brightness.
    pub fn set_brightness(&mut self, brightness: Brightness) -> Result<(), DisplayError> {
        Command::SecondPreChargePeriod(brightness.precharge).send(&mut self.interface)?;
        Command::Contrast(brightness.contrast).send(&mut self.interface)
    }

    /// Turn the display on or off. The display can be drawn to and retains all
    /// of its memory even while off.
    pub fn set_display_on(&mut self, on: bool) -> Result<(), DisplayError> {
        Command::DisplayOn(on).send(&mut self.interface)
    }

    /// Set the position in the framebuffer of the display limiting where any sent data should be
    /// drawn. This method can be used for changing the affected area on the screen as well
    /// as (re-)setting the start point of the next `draw` call.
    pub fn set_draw_area(&mut self, start: (u8, u8), end: (u8, u8)) -> Result<(), DisplayError> {
        Command::ColumnAddress(start.0.into(), end.0.saturating_sub(1)).send(&mut self.interface)?;
        Command::RowAddress(start.1.into(), end.1.saturating_sub(1)).send(&mut self.interface)?;
        
        Ok(())
    }

    /// Set the column address in the framebuffer of the display where any sent data should be
    /// drawn.
    pub fn set_column(&mut self, column: u8) -> Result<(), DisplayError> {
        println!("set_column: {}", column);
        Command::ColumnAddress(column, SIZE::WIDTH - column).send(&mut self.interface)
    }

    /// Set the page address (row 8px high) in the framebuffer of the display where any sent data
    /// should be drawn.
    ///
    /// Note that the parameter is in pixels, but the page will be set to the start of the 8px
    /// row which contains the passed-in row.
    pub fn set_row(&mut self, row: u8) -> Result<(), DisplayError> {
        println!("set_row: {}", row);
        Command::RowAddress(row, SIZE::HEIGHT - row).send(&mut self.interface)
    }

    fn flush_buffer_chunks(
        interface: &mut DI,
        buffer: &[u8],
        disp_width: usize,
        upper_left: (u8, u8),
        lower_right: (u8, u8),
    ) -> Result<(), DisplayError> {
        // Divide by 8 since each row is actually 8 pixels tall
        let num_pages = ((lower_right.1 - upper_left.1) / 8) as usize + 1;

        // Each page is 8 bits tall, so calculate which page number to start at (rounded down) from
        // the top of the display
        let starting_page = (upper_left.1 / 8) as usize;

        // Calculate start and end X coordinates for each page
        let page_lower = upper_left.0 as usize;
        let page_upper = lower_right.0 as usize;

        buffer
            .chunks(disp_width)
            .skip(starting_page)
            .take(num_pages)
            .map(|s| &s[page_lower..page_upper])
            .try_for_each(|c| interface.send_data(U8(&c)))
    }
}

// SPI-only reset
impl<SPI, DC, SIZE, MODE> Ssd1327<SPIInterfaceNoCS<SPI, DC>, SIZE, MODE> {
    /// Reset the display.
    pub fn reset<RST, DELAY, PinE>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<(), PinE>>
    where
        RST: OutputPin<Error = PinE>,
        DELAY: DelayMs<u8>,
    {
        inner_reset(rst, delay)
    }
}

// SPI-only reset
impl<SPI, DC, CS, SIZE, MODE> Ssd1327<SPIInterface<SPI, DC, CS>, SIZE, MODE> {
    /// Reset the display.
    pub fn reset<RST, DELAY, PinE>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<(), PinE>>
    where
        RST: OutputPin<Error = PinE>,
        DELAY: DelayMs<u8>,
    {
        inner_reset(rst, delay)
    }
}

fn inner_reset<RST, DELAY, PinE>(rst: &mut RST, delay: &mut DELAY) -> Result<(), Error<(), PinE>>
where
    RST: OutputPin<Error = PinE>,
    DELAY: DelayMs<u8>,
{
    rst.set_high().map_err(Error::Pin)?;
    delay.delay_ms(1);
    rst.set_low().map_err(Error::Pin)?;
    delay.delay_ms(10);
    rst.set_high().map_err(Error::Pin)
}
