//! Display commands.

use display_interface::{DataFormat::U8, DisplayError, WriteOnlyDataCommand};

/// SSD1327 Commands

/// Commands
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Command {
    /// Command 0x15
    /// Setup column start and end address
    /// values range from 0-63
    /// This is only for horizontal or vertical addressing mode
    ColumnAddress(u8, u8),
    /// Command 0x75
    /// Setup row start and end address
    /// values range from 0-127
    /// This is only for horizontal or vertical addressing mode
    RowAddress(u8, u8),
    /// Command 0x81
    /// Set contrast. Higher number is higher contrast. Default = 0x7F
    /// values range from 0x00-0xFF
    Contrast(u8),
    /// Command 0x84 ~ 0x86; 0xB2
    /// NOOP 
    Noop,
    /// Command 0xA0
    /// This command has multiple configurations
    /// values range from 0x00-0x7F
    Remap(u8),
    /// Command 0xA1
    /// Set display start line
    /// values range from 0-0x57
    DisplayStartLine(u8),
    /// Command 0xA2
    /// Set display offset
    /// values range from 0-0x3F
    DisplayOffset(u8),
    /// Command 0xA4 ~ 0xA7
    /// Set Display Mode
    DisplayMode(DisplayMode),
    /// Command 0xA8
    /// Set Multiplex ratio
    /// values range from 0x10-0x80, RESET = 0x80
    MultiplexRatio(u8),
    /// Command 0xAB
    /// Function selection A, used to enable/disable Vdd regulator
    /// values range from 0/1
    VoltageRegulator(bool),
    /// Command 0xAE/0xAF
    /// Display ON/OFF
    DisplayOn(bool),
    /// Command 0xB1
    /// Set Phase Length
    PhaseLength(PhaseLength),
    /// Command 0xB3
    /// Set Display Clock Divide Ratio/oscillator frequency
    /// 80Hz:0xc1 90Hz:0xe1   100Hz:0x00   110Hz:0x30 120Hz:0x50   130Hz:0x70     01
    FrontClockDivideRatio(FrontClockDivideRatio),
    /// Command 0xB5
    /// Set GPIO0 and GPIO1 pins output level
    SetGPIO(u8),
    /// Command 0xB6
    /// Set Second Pre-charge Period
    /// values range from 0-15
    SecondPreChargePeriod(u8),
    /// Command 0xB8
    /// Set Gray Scale Table
    /// values range from 0-15
    GrayScaleTable(u8),
    /// Command 0xB9
    /// Select Default Linear Gray Scale Table
    /// values range from 0-255
    DefaultLinearGrayScaleTable,
    /// Command 0xBC
    /// Set Pre-charge Voltage Level
    PreChargeVoltage(PreChargeVoltageLevel),
    /// Command 0xBE
    /// Set VCOMH Voltage
    /// values range from 0-255
    VcomhVoltage(VcomhLevel),
    /// Command 0xD5
    /// Function selection B
    /// values range from 0-255
    FunctionSelectionB(u8),
    /// Command 0xFD
    /// Set Command Lock
    CommandLock(bool),
    /// Command 0x26/0x27; Left/Right
    /// Horizontal Scroll Setup
    /// Values are scroll direction, start page, end page,
    /// and number of frames per step.
    HorizontalScrollSetup(HScrollDir, u8, u8, u8, u8, NFrames),
    /// Command 0x2E/0x2F
    /// Deactivate Scroll/Activate Scroll
    EnableScroll(bool)
}

impl Command {
    /// Send command to SSD1327
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        // Transform command into a fixed size array of 7 u8 and the real length for sending
        let (data, len) = match self {
            Command::ColumnAddress(start, end) => ([0x15, start, end, 0, 0, 0, 0], 3),
            Command::RowAddress(start, end) => ([0x75, start, end, 0, 0, 0, 0], 3),    
            Command::Contrast(val) => ([0x81, val, 0, 0, 0, 0, 0], 2),
            Command::Remap(val) => ([0xA0, val, 0, 0, 0, 0, 0], 2),
            Command::DisplayStartLine(val) => ([0xA1, val, 0, 0, 0, 0, 0], 2),
            Command::DisplayOffset(val) => ([0xA2, val, 0, 0, 0, 0, 0], 2),
            Command::DisplayMode(val) => ([(0xA7 & (val as u8)) | 0xA4, 0, 0, 0, 0, 0, 0], 1),
            Command::MultiplexRatio(val) => ([0xA8, val, 0, 0, 0, 0, 0], 2),
            Command::VoltageRegulator(val) => ([0xAB, val as u8, 0, 0, 0, 0, 0], 2),
            Command::DisplayOn(on) => ([0xAE | (on as u8), 0, 0, 0, 0, 0, 0], 1),
            Command::PhaseLength(val) => ([0xB1, val as u8, 0, 0, 0, 0, 0], 2),
            Command::FrontClockDivideRatio(val) => ([0xB3, val as u8, 0, 0, 0, 0, 0], 2),
            Command::SetGPIO(val) => ([0xB5, val, 0, 0, 0, 0, 0], 2),
            Command::SecondPreChargePeriod(val) => ([0xB6, val, 0, 0, 0, 0, 0], 2),
            Command::GrayScaleTable(val) => ([0xB8, val & 0xF, 0, 0, 0, 0, 0], 2),
            Command::DefaultLinearGrayScaleTable => ([0xB9, 0, 0, 0, 0, 0, 0], 1),
            Command::PreChargeVoltage(val) => ([0xBC, val as u8, 0, 0, 0, 0, 0], 2),
            Command::VcomhVoltage(val) => ([0xBE, val as u8, 0, 0, 0, 0, 0], 2),
            Command::FunctionSelectionB(val) => ([0xD5, val, 0, 0, 0, 0, 0], 2),
            Command::CommandLock(val) => ([0xFD, ((val as u8) << 2) | 0x12, 0, 0, 0, 0, 0], 2),
            Command::HorizontalScrollSetup(
                dir, 
                col_start, 
                col_end,
                row_start, 
                row_end,  
                frames
            ) => (
                [
                    0x26 | (dir as u8),
                    0,
                    row_start & 0x7F,
                    frames as u8,
                    row_end & 0x7F,
                    col_start & 0x3F,
                    col_end & 0x3F,
                ],
                7
            ),
            Command::EnableScroll(enable) => ([0x2E | (enable as u8), 0, 0, 0, 0, 0, 0], 1),
            Command::Noop => todo!(),
        };

        // Send command over the interface
        iface.send_commands(U8(&data[0..len]))
    }
}

/// Horizontal Scroll Direction
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum HScrollDir {
    /// Left to right
    LeftToRight = 0,
    /// Right to left
    RightToLeft = 1,
}

/// display clock hz
#[derive(Debug, Clone, Copy)]
pub enum FrontClockDivideRatio {
    /// 80Hz
    Hz80 = 0xC1,
    /// 90Hz
    Hz90 = 0xE1,
    /// 100Hz
    Hz100 = 0x00,
    /// 110Hz
    Hz110 = 0x30,
    /// 120Hz
    Hz120 = 0x50,
    /// 130Hz
    Hz130 = 0x70,
}

/// Frame interval
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum NFrames {
    /// 2 Frames
    F2 = 0b111,
    /// 3 Frames
    F3 = 0b100,
    /// 4 Frames
    F4 = 0b101,
    /// 5 Frames
    F5 = 0b110,
    /// 6 Frames
    F6 = 0b000,
    /// 32 Frames
    F32 = 0b001,
    /// 64 Frames
    F64 = 0b010,
    /// 256 Frames
    F256 = 0b011
}

/// Vcomh Deselect level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum VcomhLevel {
    /// 0.72 * Vcc
    V072 = 0b000,
    /// 0.82 * Vcc
    V082 = 0b101,
    /// 0.86 * Vcc
    V086 = 0b111
}

/// Display Mode
#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    /// Normal mode
    Normal = 0xA4,
    /// Set Entire Display On
    AllOn = 0xA5,
    /// Set Entire Display Off
    AllOff = 0xA6,
    /// Invert mode
    Invert = 0xA7
}

/// Period of Phase 1 and 2
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum PhaseLength {
    /// Phase 1: reset
    PhaseReset = 0x0F,
    /// Phase 2: Precharge
    PhasePreCharge = 0xF0,
    /// Phase Auto
    PhaseAuto = 0xF1
}

/// Pre-charge voltage level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum PreChargeVoltageLevel {
    /// 0.20 * Vcc
    V020 = 0x00,
    /// 0.50 * Vcc
    V050 = 0x05,
    /// 0.613 * Vcc
    V0613 = 0x07,
    /// Vcomh
    Vcomh = 0x08
}

/// Generate a Remap Configuration
pub fn generate_remap (
    col_remap: bool, 
    nibble_remap: bool,
    dir_increment: bool,
    vertical_scale: bool,
    com_remap: bool,
    horizontal_scale: bool,
    odd_even_com_split: bool,
) -> u8 {
    0x7F & (
        (col_remap as u8) << 0 |
        (nibble_remap as u8) << 1 |
        (dir_increment as u8) << 2 |
        (vertical_scale as u8) << 3 |
        (com_remap as u8) << 4 |
        (horizontal_scale as u8) << 5 |
        (odd_even_com_split as u8) << 6
    )
}