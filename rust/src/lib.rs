//! # ADS1263 Rust Driver
//!
//! A Rust driver for the Texas Instruments ADS1263 32-bit, high-precision
//! delta-sigma ADC, designed for use with the Waveshare High-Precision AD HAT
//! on Raspberry Pi.
//!
//! ## Features
//!
//! - Full support for ADC1 (32-bit) and ADC2 (24-bit)
//! - Single-ended and differential input modes
//! - Configurable gain, data rate, and digital filters
//! - RTD (Resistance Temperature Detector) measurement support
//! - DAC output control
//! - Automatic CRC checksum verification
//!
//! ## Hardware Requirements
//!
//! - Raspberry Pi (tested on Pi 4, should work on Pi 3/5)
//! - Waveshare High-Precision AD HAT (or compatible ADS1263 board)
//! - SPI enabled (`sudo raspi-config` -> Interface Options -> SPI)
//!
//! ## Quick Start
//!
//! ```no_run
//! use ads1263::{Ads1263, Hal, DataRate, InputMode};
//!
//! fn main() -> ads1263::Result<()> {
//!     // Initialize hardware
//!     let hal = Hal::new()?;
//!     let mut adc = Ads1263::new(hal);
//!
//!     // Configure for single-ended measurements
//!     adc.set_mode(InputMode::SingleEnded);
//!     adc.init_adc1(DataRate::Sps400)?;
//!
//!     // Read channel 0
//!     let raw = adc.get_channel_value(0)?;
//!     let voltage = Ads1263::raw_to_voltage_adc1(raw, 5.0);
//!
//!     println!("Channel 0: {:.6} V", voltage);
//!     Ok(())
//! }
//! ```
//!
//! ## Pin Connections
//!
//! | AD HAT | Raspberry Pi (BCM) | Function |
//! |--------|-------------------|----------|
//! | VCC    | 3.3V              | Power    |
//! | GND    | GND               | Ground   |
//! | DIN    | GPIO 10           | SPI MOSI |
//! | DOUT   | GPIO 9            | SPI MISO |
//! | SCLK   | GPIO 11           | SPI CLK  |
//! | CS     | GPIO 22           | Chip Select |
//! | DRDY   | GPIO 17           | Data Ready |
//! | RST    | GPIO 18           | Reset    |
//!
//! ## Error Handling
//!
//! All fallible operations return `Result<T, Ads1263Error>`. The error type
//! provides detailed information about what went wrong:
//!
//! ```no_run
//! use ads1263::{Ads1263, Hal, DataRate, Ads1263Error};
//!
//! fn main() {
//!     let result = Hal::new();
//!     match result {
//!         Ok(hal) => {
//!             let mut adc = Ads1263::new(hal);
//!             // Use ADC...
//!         }
//!         Err(Ads1263Error::Gpio(e)) => {
//!             eprintln!("GPIO error: {}. Is SPI enabled?", e);
//!         }
//!         Err(e) => {
//!             eprintln!("Error: {}", e);
//!         }
//!     }
//! }
//! ```

pub mod ads1263;
pub mod error;
pub mod hal;
pub mod registers;

// Re-export main types for convenience
pub use ads1263::Ads1263;
pub use error::{Ads1263Error, Result};
pub use hal::{Hal, PinConfig, SpiConfig};
pub use registers::{
    Adc2DataRate, Adc2Gain, Command, DacVoltage, DataRate, Delay, DigitalFilter, Gain, InputMode,
    ReferenceSource, Register,
};
