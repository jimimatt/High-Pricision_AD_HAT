//! Error types for the ADS1263 driver

use thiserror::Error;

/// Errors that can occur when interacting with the ADS1263
#[derive(Error, Debug)]
pub enum Ads1263Error {
    /// SPI communication error
    #[error("SPI error: {0}")]
    Spi(#[from] rppal::spi::Error),

    /// GPIO error
    #[error("GPIO error: {0}")]
    Gpio(#[from] rppal::gpio::Error),

    /// Device initialization failed
    #[error("Device initialization failed")]
    InitFailed,

    /// Invalid chip ID detected
    #[error("Invalid chip ID: expected 1, got {0}")]
    InvalidChipId(u8),

    /// Invalid channel number specified
    #[error("Invalid channel: {0} (max: {1})")]
    InvalidChannel(u8, u8),

    /// Timeout waiting for DRDY signal
    #[error("Timeout waiting for DRDY")]
    Timeout,

    /// CRC checksum verification failed
    #[error("CRC checksum error")]
    ChecksumError,

    /// Register write verification failed
    #[error("Register write verification failed for {register}")]
    RegisterVerifyFailed { register: &'static str },
}

/// Result type alias for ADS1263 operations
pub type Result<T> = std::result::Result<T, Ads1263Error>;
