//! Display modes.

mod buffered_graphics;
mod terminal;

use crate::{rotation::DisplayRotation, size::DisplaySize, Ssd1327};
pub use buffered_graphics::*;
use display_interface::{DisplayError, WriteOnlyDataCommand};
pub use terminal::*;

/// Common functions to all display modes.
pub trait DisplayConfig {
    /// Error.
    type Error;

    /// Set display rotation.
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initialise and configure the display for the given mode.
    fn init(&mut self) -> Result<(), Self::Error>;
}

/// A mode with no additional functionality beyond that provided by the base [`Ssd1327`] struct.
#[derive(Debug, Copy, Clone)]
pub struct BasicMode;

impl<DI, SIZE> Ssd1327<DI, SIZE, BasicMode>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    /// Clear the display.
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        println!("basicMode clear");
        self.set_draw_area((1, 1), (128, 128))?;

        self.draw(&[0u8; 2048])
    }
}

impl<DI, SIZE> DisplayConfig for Ssd1327<DI, SIZE, BasicMode>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    type Error = DisplayError;

    /// Set the display rotation.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_rotation(rot)
    }

    /// Initialise in horizontal addressing mode.
    fn init(&mut self) -> Result<(), DisplayError> {
        println!("basicMode init");
        self.init_basic()
    }
}
