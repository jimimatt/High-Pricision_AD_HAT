//! Hardware Abstraction Layer for ADS1263
//!
//! This module provides low-level hardware access for GPIO and SPI
//! communication with the ADS1263 ADC on Raspberry Pi.

use crate::error::{Ads1263Error, Result};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::thread;
use std::time::Duration;

/// Pin configuration for the ADS1263 HAT
///
/// Default pins match the Waveshare High-Precision AD HAT
#[derive(Debug, Clone, Copy)]
pub struct PinConfig {
    /// Reset pin (BCM numbering)
    pub rst: u8,
    /// Chip select pin (BCM numbering)
    pub cs: u8,
    /// Data ready pin (BCM numbering)
    pub drdy: u8,
}

impl Default for PinConfig {
    fn default() -> Self {
        Self {
            rst: 18,  // BCM pin 18
            cs: 22,   // BCM pin 22
            drdy: 17, // BCM pin 17
        }
    }
}

/// SPI configuration
#[derive(Debug, Clone, Copy)]
pub struct SpiConfig {
    /// SPI bus (usually Spi0)
    pub bus: Bus,
    /// Slave select (usually Ss0)
    pub slave_select: SlaveSelect,
    /// Clock speed in Hz
    pub clock_speed: u32,
    /// SPI mode (Mode1 for ADS1263: CPOL=0, CPHA=1)
    pub mode: Mode,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            bus: Bus::Spi0,
            slave_select: SlaveSelect::Ss0,
            clock_speed: 1_000_000, // 1 MHz
            mode: Mode::Mode1,      // CPOL=0, CPHA=1
        }
    }
}

/// Hardware Abstraction Layer
///
/// Provides low-level access to GPIO and SPI for the ADS1263
pub struct Hal {
    spi: Spi,
    rst_pin: OutputPin,
    cs_pin: OutputPin,
    drdy_pin: InputPin,
}

impl Hal {
    /// Create a new HAL instance with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if GPIO or SPI initialization fails
    pub fn new() -> Result<Self> {
        Self::with_config(PinConfig::default(), SpiConfig::default())
    }

    /// Create a new HAL instance with custom pin configuration
    ///
    /// # Errors
    ///
    /// Returns an error if GPIO or SPI initialization fails
    pub fn with_pins(pin_config: PinConfig) -> Result<Self> {
        Self::with_config(pin_config, SpiConfig::default())
    }

    /// Create a new HAL instance with full custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if GPIO or SPI initialization fails
    pub fn with_config(pin_config: PinConfig, spi_config: SpiConfig) -> Result<Self> {
        let gpio = Gpio::new()?;

        // Configure GPIO pins
        let rst_pin = gpio.get(pin_config.rst)?.into_output();
        let cs_pin = gpio.get(pin_config.cs)?.into_output_high(); // CS starts high (inactive)
        let drdy_pin = gpio.get(pin_config.drdy)?.into_input();

        // Configure SPI
        let spi = Spi::new(
            spi_config.bus,
            spi_config.slave_select,
            spi_config.clock_speed,
            spi_config.mode,
        )?;

        log::info!(
            "HAL initialized - RST: BCM{}, CS: BCM{}, DRDY: BCM{}",
            pin_config.rst,
            pin_config.cs,
            pin_config.drdy
        );
        log::info!(
            "SPI configured - Bus: {:?}, Speed: {} Hz, Mode: {:?}",
            spi_config.bus,
            spi_config.clock_speed,
            spi_config.mode
        );

        Ok(Self {
            spi,
            rst_pin,
            cs_pin,
            drdy_pin,
        })
    }

    /// Set the reset pin state
    ///
    /// # Arguments
    ///
    /// * `high` - true to set pin high, false to set pin low
    #[inline]
    pub fn set_rst(&mut self, high: bool) {
        if high {
            self.rst_pin.set_high();
        } else {
            self.rst_pin.set_low();
        }
    }

    /// Set the chip select pin state
    ///
    /// # Arguments
    ///
    /// * `high` - true to set pin high (inactive), false to set pin low (active)
    #[inline]
    pub fn set_cs(&mut self, high: bool) {
        if high {
            self.cs_pin.set_high();
        } else {
            self.cs_pin.set_low();
        }
    }

    /// Read the data ready pin state
    ///
    /// # Returns
    ///
    /// true if DRDY is high (not ready), false if low (data ready)
    #[inline]
    pub fn read_drdy(&self) -> bool {
        self.drdy_pin.is_high()
    }

    /// Transfer a single byte over SPI (simultaneous write and read)
    ///
    /// # Arguments
    ///
    /// * `tx` - Byte to transmit
    ///
    /// # Returns
    ///
    /// The byte received during transmission
    ///
    /// # Errors
    ///
    /// Returns an error if SPI transfer fails
    #[inline]
    pub fn spi_transfer_byte(&mut self, tx: u8) -> Result<u8> {
        let write_buffer = [tx];
        let mut read_buffer = [0u8];
        self.spi.transfer(&mut read_buffer, &write_buffer)?;
        Ok(read_buffer[0])
    }

    /// Write a byte over SPI (discard response)
    ///
    /// # Arguments
    ///
    /// * `tx` - Byte to transmit
    ///
    /// # Errors
    ///
    /// Returns an error if SPI transfer fails
    #[inline]
    pub fn spi_write_byte(&mut self, tx: u8) -> Result<()> {
        self.spi_transfer_byte(tx)?;
        Ok(())
    }

    /// Read a byte over SPI (send 0x00)
    ///
    /// # Returns
    ///
    /// The byte received
    ///
    /// # Errors
    ///
    /// Returns an error if SPI transfer fails
    #[inline]
    pub fn spi_read_byte(&mut self) -> Result<u8> {
        self.spi_transfer_byte(0x00)
    }

    /// Transfer multiple bytes over SPI
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer containing bytes to send; will be overwritten with received bytes
    ///
    /// # Errors
    ///
    /// Returns an error if SPI transfer fails
    pub fn spi_transfer(&mut self, buffer: &mut [u8]) -> Result<()> {
        let write_buffer = buffer.to_vec();
        self.spi.transfer(buffer, &write_buffer)?;
        Ok(())
    }

    /// Delay for a specified number of milliseconds
    ///
    /// # Arguments
    ///
    /// * `ms` - Number of milliseconds to delay
    #[inline]
    pub fn delay_ms(&self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }

    /// Delay for a specified number of microseconds
    ///
    /// # Arguments
    ///
    /// * `us` - Number of microseconds to delay
    #[inline]
    pub fn delay_us(&self, us: u64) {
        thread::sleep(Duration::from_micros(us));
    }

    /// Wait for DRDY to go low (data ready) with timeout
    ///
    /// The ADS1263 pulls DRDY low when new conversion data is available.
    ///
    /// # Errors
    ///
    /// Returns `Ads1263Error::Timeout` if DRDY doesn't go low within the timeout period
    pub fn wait_drdy(&self) -> Result<()> {
        const TIMEOUT_ITERATIONS: u32 = 4_000_000;

        for _ in 0..TIMEOUT_ITERATIONS {
            if !self.read_drdy() {
                return Ok(());
            }
        }

        log::error!("Timeout waiting for DRDY");
        Err(Ads1263Error::Timeout)
    }

    /// Wait for DRDY with a specified timeout in milliseconds
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    ///
    /// # Errors
    ///
    /// Returns `Ads1263Error::Timeout` if DRDY doesn't go low within the timeout period
    pub fn wait_drdy_timeout(&self, timeout_ms: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if !self.read_drdy() {
                return Ok(());
            }
            // Small sleep to avoid busy-waiting
            thread::sleep(Duration::from_micros(10));
        }

        log::error!("Timeout ({} ms) waiting for DRDY", timeout_ms);
        Err(Ads1263Error::Timeout)
    }

    /// Perform cleanup - set control pins low
    pub fn cleanup(&mut self) {
        self.rst_pin.set_low();
        self.cs_pin.set_low();
        log::debug!("HAL cleanup completed");
    }
}

impl Drop for Hal {
    fn drop(&mut self) {
        self.cleanup();
    }
}
