//! ADS1263 32-bit Delta-Sigma ADC Driver
//!
//! This module provides the main driver implementation for the Texas Instruments
//! ADS1263 ADC, featuring:
//!
//! - 32-bit primary ADC (ADC1)
//! - 24-bit auxiliary ADC (ADC2)
//! - 10 single-ended or 5 differential input channels
//! - Programmable gain amplifier (PGA)
//! - Multiple data rates from 2.5 SPS to 38.4 kSPS
//! - Internal and external voltage references
//! - DAC outputs for sensor biasing
//! - RTD measurement support

use crate::error::{Ads1263Error, Result};
use crate::hal::Hal;
use crate::registers::*;

/// ADS1263 ADC Driver
///
/// Provides high-level interface for controlling the ADS1263 ADC.
///
/// # Example
///
/// ```no_run
/// use ads1263::{Ads1263, Hal, DataRate, InputMode};
///
/// let hal = Hal::new()?;
/// let mut adc = Ads1263::new(hal);
///
/// adc.set_mode(InputMode::SingleEnded);
/// adc.init_adc1(DataRate::Sps400)?;
///
/// let raw_value = adc.get_channel_value(0)?;
/// let voltage = Ads1263::raw_to_voltage_adc1(raw_value, 5.0);
/// println!("Channel 0: {} V", voltage);
/// # Ok::<(), ads1263::Ads1263Error>(())
/// ```
pub struct Ads1263 {
    hal: Hal,
    scan_mode: InputMode,
}

impl Ads1263 {
    // ========================================================================
    // Constructor
    // ========================================================================

    /// Create a new ADS1263 driver instance
    ///
    /// # Arguments
    ///
    /// * `hal` - Hardware abstraction layer instance
    pub fn new(hal: Hal) -> Self {
        Self {
            hal,
            scan_mode: InputMode::SingleEnded,
        }
    }

    // ========================================================================
    // Low-level operations
    // ========================================================================

    /// Hardware reset the device
    ///
    /// Performs a full hardware reset cycle using the RST pin.
    fn reset(&mut self) {
        log::debug!("Performing hardware reset");
        self.hal.set_rst(true);
        self.hal.delay_ms(300);
        self.hal.set_rst(false);
        self.hal.delay_ms(300);
        self.hal.set_rst(true);
        self.hal.delay_ms(300);
    }

    /// Send a command to the ADC
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to send
    fn write_cmd(&mut self, cmd: Command) -> Result<()> {
        self.hal.set_cs(false);
        self.hal.spi_write_byte(cmd as u8)?;
        self.hal.set_cs(true);
        Ok(())
    }

    /// Write to a register
    ///
    /// # Arguments
    ///
    /// * `reg` - Register to write to
    /// * `data` - Data byte to write
    fn write_reg(&mut self, reg: Register, data: u8) -> Result<()> {
        self.hal.set_cs(false);
        self.hal.spi_write_byte(Command::WReg as u8 | reg as u8)?;
        self.hal.spi_write_byte(0x00)?; // Number of registers to write minus 1
        self.hal.spi_write_byte(data)?;
        self.hal.set_cs(true);
        Ok(())
    }

    /// Read from a register
    ///
    /// # Arguments
    ///
    /// * `reg` - Register to read from
    ///
    /// # Returns
    ///
    /// The register value
    fn read_reg(&mut self, reg: Register) -> Result<u8> {
        self.hal.set_cs(false);
        self.hal.spi_write_byte(Command::RReg as u8 | reg as u8)?;
        self.hal.spi_write_byte(0x00)?; // Number of registers to read minus 1
        let data = self.hal.spi_read_byte()?;
        self.hal.set_cs(true);
        Ok(data)
    }

    /// Write to a register and verify the write
    ///
    /// # Arguments
    ///
    /// * `reg` - Register to write to
    /// * `data` - Data byte to write
    /// * `name` - Register name for error reporting
    fn write_reg_verify(&mut self, reg: Register, data: u8, name: &'static str) -> Result<()> {
        self.write_reg(reg, data)?;
        self.hal.delay_ms(1);

        let read_back = self.read_reg(reg)?;
        if read_back == data {
            log::info!("{} configured successfully (0x{:02X})", name, data);
            Ok(())
        } else {
            log::warn!(
                "{} configuration mismatch: wrote 0x{:02X}, read 0x{:02X}",
                name,
                data,
                read_back
            );
            Ok(()) // Continue despite mismatch (matching C behavior)
        }
    }

    // ========================================================================
    // Checksum validation
    // ========================================================================

    /// Verify checksum for ADC data
    ///
    /// The ADS1263 appends a CRC byte to each data read.
    ///
    /// # Arguments
    ///
    /// * `val` - The data value to check
    /// * `crc` - The CRC byte received from the ADC
    ///
    /// # Returns
    ///
    /// true if checksum is valid, false otherwise
    fn checksum(val: u32, crc: u8) -> bool {
        let mut sum: u8 = 0;
        let mut v = val;
        while v != 0 {
            sum = sum.wrapping_add((v & 0xFF) as u8);
            v >>= 8;
        }
        sum = sum.wrapping_add(0x9B);
        sum == crc
    }

    // ========================================================================
    // Public configuration methods
    // ========================================================================

    /// Read the chip ID
    ///
    /// # Returns
    ///
    /// The chip ID (should be 1 for ADS1263)
    pub fn read_chip_id(&mut self) -> Result<u8> {
        let id = self.read_reg(Register::Id)?;
        Ok(id >> 5)
    }

    /// Set the input mode (single-ended or differential)
    ///
    /// # Arguments
    ///
    /// * `mode` - Input mode to use
    pub fn set_mode(&mut self, mode: InputMode) {
        self.scan_mode = mode;
        log::info!("Input mode set to {:?}", mode);
    }

    /// Get the current input mode
    pub fn get_mode(&self) -> InputMode {
        self.scan_mode
    }

    // ========================================================================
    // ADC1 Configuration
    // ========================================================================

    /// Configure ADC1 with specified parameters
    ///
    /// # Arguments
    ///
    /// * `gain` - PGA gain setting
    /// * `drate` - Data rate setting
    /// * `delay` - Conversion delay setting
    fn config_adc1(&mut self, gain: Gain, drate: DataRate, delay: Delay) -> Result<()> {
        // MODE2: PGA bypassed (0x80) | gain | data rate
        let mode2 = 0x80 | ((gain as u8) << 4) | (drate as u8);
        self.write_reg_verify(Register::Mode2, mode2, "REG_MODE2")?;

        // REFMUX: VDD, VSS as reference (0x24)
        let refmux = ReferenceSource::AvddAvss as u8;
        self.write_reg_verify(Register::RefMux, refmux, "REG_REFMUX")?;

        // MODE0: Conversion delay
        let mode0 = delay as u8;
        self.write_reg_verify(Register::Mode0, mode0, "REG_MODE0")?;

        // MODE1: Digital filter - FIR (0x84)
        let mode1 = DigitalFilter::Fir as u8;
        self.write_reg_verify(Register::Mode1, mode1, "REG_MODE1")?;

        Ok(())
    }

    /// Initialize ADC1 with specified data rate
    ///
    /// Performs hardware reset, verifies chip ID, and configures ADC1.
    ///
    /// # Arguments
    ///
    /// * `rate` - Data rate setting
    ///
    /// # Errors
    ///
    /// Returns `Ads1263Error::InvalidChipId` if chip ID is not 1
    pub fn init_adc1(&mut self, rate: DataRate) -> Result<()> {
        self.reset();

        let chip_id = self.read_chip_id()?;
        if chip_id == 1 {
            log::info!("Chip ID verified: {}", chip_id);
        } else {
            log::error!("Invalid chip ID: {} (expected 1)", chip_id);
            return Err(Ads1263Error::InvalidChipId(chip_id));
        }

        self.write_cmd(Command::Stop1)?;
        self.config_adc1(Gain::Gain1, rate, Delay::Delay35us)?;
        self.write_cmd(Command::Start1)?;

        log::info!("ADC1 initialized with data rate {:?}", rate);
        Ok(())
    }

    // ========================================================================
    // ADC2 Configuration
    // ========================================================================

    /// Configure ADC2 with specified parameters
    fn config_adc2(&mut self, gain: Adc2Gain, drate: Adc2DataRate, delay: Delay) -> Result<()> {
        // ADC2CFG: VAVDD/VAVSS reference (0x20) | data rate | gain
        let adc2cfg = 0x20 | ((drate as u8) << 6) | (gain as u8);
        self.write_reg_verify(Register::Adc2Cfg, adc2cfg, "REG_ADC2CFG")?;

        // MODE0: Conversion delay
        let mode0 = delay as u8;
        self.write_reg_verify(Register::Mode0, mode0, "REG_MODE0")?;

        Ok(())
    }

    /// Initialize ADC2 with specified data rate
    ///
    /// # Arguments
    ///
    /// * `rate` - Data rate setting
    ///
    /// # Errors
    ///
    /// Returns `Ads1263Error::InvalidChipId` if chip ID is not 1
    pub fn init_adc2(&mut self, rate: Adc2DataRate) -> Result<()> {
        self.reset();

        let chip_id = self.read_chip_id()?;
        if chip_id == 1 {
            log::info!("Chip ID verified: {}", chip_id);
        } else {
            log::error!("Invalid chip ID: {} (expected 1)", chip_id);
            return Err(Ads1263Error::InvalidChipId(chip_id));
        }

        self.write_cmd(Command::Stop2)?;
        self.config_adc2(Adc2Gain::Gain1, rate, Delay::Delay35us)?;

        log::info!("ADC2 initialized with data rate {:?}", rate);
        Ok(())
    }

    // ========================================================================
    // Channel selection
    // ========================================================================

    /// Set single-ended input channel for ADC1
    fn set_channel(&mut self, channel: u8) -> Result<()> {
        if channel > 10 {
            return Err(Ads1263Error::InvalidChannel(channel, 10));
        }
        // INPMUX: channel as positive, VCOM (0x0A) as negative
        let inpmux = (channel << 4) | 0x0A;
        self.write_reg(Register::InpMux, inpmux)?;
        Ok(())
    }

    /// Set differential input channel for ADC1
    fn set_diff_channel(&mut self, channel: u8) -> Result<()> {
        let inpmux = match channel {
            0 => (0 << 4) | 1, // AIN0 - AIN1
            1 => (2 << 4) | 3, // AIN2 - AIN3
            2 => (4 << 4) | 5, // AIN4 - AIN5
            3 => (6 << 4) | 7, // AIN6 - AIN7
            4 => (8 << 4) | 9, // AIN8 - AIN9
            _ => return Err(Ads1263Error::InvalidChannel(channel, 4)),
        };
        self.write_reg(Register::InpMux, inpmux)?;
        Ok(())
    }

    /// Set single-ended input channel for ADC2
    fn set_channel_adc2(&mut self, channel: u8) -> Result<()> {
        if channel > 10 {
            return Err(Ads1263Error::InvalidChannel(channel, 10));
        }
        let inpmux = (channel << 4) | 0x0A;
        self.write_reg(Register::Adc2Mux, inpmux)?;
        Ok(())
    }

    /// Set differential input channel for ADC2
    fn set_diff_channel_adc2(&mut self, channel: u8) -> Result<()> {
        let inpmux = match channel {
            0 => (0 << 4) | 1,
            1 => (2 << 4) | 3,
            2 => (4 << 4) | 5,
            3 => (6 << 4) | 7,
            4 => (8 << 4) | 9,
            _ => return Err(Ads1263Error::InvalidChannel(channel, 4)),
        };
        self.write_reg(Register::Adc2Mux, inpmux)?;
        Ok(())
    }

    // ========================================================================
    // Data reading
    // ========================================================================

    /// Read raw ADC1 data (32-bit)
    fn read_adc1_data(&mut self) -> Result<u32> {
        self.hal.set_cs(false);

        // Wait for valid status
        let _status = loop {
            self.hal.spi_write_byte(Command::RData1 as u8)?;
            let s = self.hal.spi_read_byte()?;
            if (s & 0x40) != 0 {
                break s;
            }
        };

        // Read 4 data bytes + CRC
        let b0 = self.hal.spi_read_byte()?;
        let b1 = self.hal.spi_read_byte()?;
        let b2 = self.hal.spi_read_byte()?;
        let b3 = self.hal.spi_read_byte()?;
        let crc = self.hal.spi_read_byte()?;

        self.hal.set_cs(true);

        let data =
            ((b0 as u32) << 24) | ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        if !Self::checksum(data, crc) {
            log::warn!("ADC1 checksum error: data=0x{:08X}, crc=0x{:02X}", data, crc);
        }

        Ok(data)
    }

    /// Read raw ADC2 data (24-bit)
    fn read_adc2_data(&mut self) -> Result<u32> {
        self.hal.set_cs(false);

        // Wait for valid status
        loop {
            self.hal.spi_write_byte(Command::RData2 as u8)?;
            let s = self.hal.spi_read_byte()?;
            if (s & 0x80) != 0 {
                break;
            }
        }

        // Read 3 data bytes + padding + CRC (ADC2 is 24-bit)
        let b0 = self.hal.spi_read_byte()?;
        let b1 = self.hal.spi_read_byte()?;
        let b2 = self.hal.spi_read_byte()?;
        let _b3 = self.hal.spi_read_byte()?; // Padding
        let crc = self.hal.spi_read_byte()?;

        self.hal.set_cs(true);

        let data = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        if !Self::checksum(data, crc) {
            log::warn!("ADC2 checksum error: data=0x{:06X}, crc=0x{:02X}", data, crc);
        }

        Ok(data)
    }

    // ========================================================================
    // Public data acquisition methods
    // ========================================================================

    /// Get ADC1 channel value (raw 32-bit)
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number (0-10 for single-ended, 0-4 for differential)
    ///
    /// # Returns
    ///
    /// Raw 32-bit ADC value
    pub fn get_channel_value(&mut self, channel: u8) -> Result<u32> {
        match self.scan_mode {
            InputMode::SingleEnded => {
                if channel > 10 {
                    return Err(Ads1263Error::InvalidChannel(channel, 10));
                }
                self.set_channel(channel)?;
            }
            InputMode::Differential => {
                if channel > 4 {
                    return Err(Ads1263Error::InvalidChannel(channel, 4));
                }
                self.set_diff_channel(channel)?;
            }
        }

        self.hal.wait_drdy()?;
        self.read_adc1_data()
    }

    /// Get ADC2 channel value (raw 24-bit)
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number (0-10 for single-ended, 0-4 for differential)
    ///
    /// # Returns
    ///
    /// Raw 24-bit ADC value (stored in u32)
    pub fn get_channel_value_adc2(&mut self, channel: u8) -> Result<u32> {
        match self.scan_mode {
            InputMode::SingleEnded => {
                if channel > 10 {
                    return Err(Ads1263Error::InvalidChannel(channel, 10));
                }
                self.set_channel_adc2(channel)?;
            }
            InputMode::Differential => {
                if channel > 4 {
                    return Err(Ads1263Error::InvalidChannel(channel, 4));
                }
                self.set_diff_channel_adc2(channel)?;
            }
        }

        self.write_cmd(Command::Start2)?;
        self.read_adc2_data()
    }

    /// Read multiple channels from ADC1
    ///
    /// # Arguments
    ///
    /// * `channels` - Slice of channel numbers to read
    ///
    /// # Returns
    ///
    /// Vector of raw 32-bit values in the same order as input channels
    pub fn get_all(&mut self, channels: &[u8]) -> Result<Vec<u32>> {
        let mut values = Vec::with_capacity(channels.len());
        for &ch in channels {
            values.push(self.get_channel_value(ch)?);
        }
        Ok(values)
    }

    /// Read all 10 channels from ADC2
    ///
    /// # Returns
    ///
    /// Array of 10 raw 24-bit values
    pub fn get_all_adc2(&mut self) -> Result<[u32; 10]> {
        let mut values = [0u32; 10];
        for i in 0..10 {
            values[i] = self.get_channel_value_adc2(i as u8)?;
            self.write_cmd(Command::Stop2)?;
        }
        Ok(values)
    }

    // ========================================================================
    // Voltage conversion utilities
    // ========================================================================

    /// Convert raw ADC1 value to voltage
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw 32-bit ADC value
    /// * `reference` - Reference voltage in volts
    ///
    /// # Returns
    ///
    /// Voltage in volts (can be negative for differential measurements)
    pub fn raw_to_voltage_adc1(raw: u32, reference: f64) -> f64 {
        if (raw >> 31) == 1 {
            // Negative value (MSB set)
            -(reference * 2.0 - (raw as f64 / 2147483648.0) * reference)
        } else {
            // Positive value
            (raw as f64 / 2147483647.0) * reference
        }
    }

    /// Convert raw ADC2 value to voltage
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw 24-bit ADC value
    /// * `reference` - Reference voltage in volts
    ///
    /// # Returns
    ///
    /// Voltage in volts (can be negative for differential measurements)
    pub fn raw_to_voltage_adc2(raw: u32, reference: f64) -> f64 {
        if (raw >> 23) == 1 {
            // Negative value (MSB set)
            -(reference * 2.0 - (raw as f64 / 8388608.0) * reference)
        } else {
            // Positive value
            (raw as f64 / 8388607.0) * reference
        }
    }

    // ========================================================================
    // RTD (Resistance Temperature Detector) support
    // ========================================================================

    /// Configure and read RTD measurement
    ///
    /// Sets up the ADC for RTD measurement using the internal current sources.
    ///
    /// # Arguments
    ///
    /// * `delay` - Conversion delay
    /// * `gain` - PGA gain
    /// * `drate` - Data rate
    ///
    /// # Returns
    ///
    /// Raw 32-bit ADC value
    pub fn read_rtd(&mut self, delay: Delay, gain: Gain, drate: DataRate) -> Result<u32> {
        // MODE0 (CHOP OFF)
        self.write_reg(Register::Mode0, delay as u8)?;
        self.hal.delay_ms(1);

        // IDACMUX: IDAC2 to AINCOM, IDAC1 to AIN3
        self.write_reg(Register::IdacMux, (0x0A << 4) | 0x03)?;
        self.hal.delay_ms(1);

        // IDACMAG: IDAC2 = IDAC1 = 250µA
        self.write_reg(Register::IdacMag, (0x03 << 4) | 0x03)?;
        self.hal.delay_ms(1);

        // MODE2: gain | data rate
        let mode2 = ((gain as u8) << 4) | (drate as u8);
        self.write_reg(Register::Mode2, mode2)?;
        self.hal.delay_ms(1);

        // INPMUX: AINP = AIN7, AINN = AIN6
        self.write_reg(Register::InpMux, (0x07 << 4) | 0x06)?;
        self.hal.delay_ms(1);

        // REFMUX: AIN4, AIN5
        self.write_reg(Register::RefMux, (0x03 << 3) | 0x03)?;
        self.hal.delay_ms(1);

        // Read one conversion
        self.write_cmd(Command::Start1)?;
        self.hal.delay_ms(10);
        self.hal.wait_drdy()?;
        let value = self.read_adc1_data()?;
        self.write_cmd(Command::Stop1)?;

        Ok(value)
    }

    /// Convert RTD raw value to resistance
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw ADC value from `read_rtd()`
    /// * `r_ref` - Reference resistor value in ohms (e.g., 2000.0 for 2kΩ)
    ///
    /// # Returns
    ///
    /// Resistance in ohms
    pub fn rtd_to_resistance(raw: u32, r_ref: f64) -> f64 {
        (raw as f64 / 2147483647.0) * 2.0 * r_ref
    }

    /// Convert PT100 resistance to temperature
    ///
    /// Uses simplified linear approximation for PT100 sensors.
    ///
    /// # Arguments
    ///
    /// * `resistance` - Resistance in ohms
    ///
    /// # Returns
    ///
    /// Temperature in degrees Celsius
    pub fn pt100_to_celsius(resistance: f64) -> f64 {
        // PT100 coefficient: α = 0.00385
        (resistance / 100.0 - 1.0) / 0.00385
    }

    // ========================================================================
    // DAC control
    // ========================================================================

    /// Configure DAC output
    ///
    /// The ADS1263 has two DAC outputs that can be used for sensor biasing.
    ///
    /// # Arguments
    ///
    /// * `voltage` - Output voltage setting
    /// * `positive` - true for positive DAC (AIN6), false for negative DAC (AIN7)
    /// * `enable` - true to enable output, false to disable
    pub fn set_dac(&mut self, voltage: DacVoltage, positive: bool, enable: bool) -> Result<()> {
        let reg = if positive {
            Register::TdacP // Controls AIN6
        } else {
            Register::TdacN // Controls AIN7
        };

        let value = if enable {
            (voltage as u8) | 0x80 // Set enable bit
        } else {
            0x00
        };

        self.write_reg(reg, value)?;
        log::debug!(
            "DAC {} {} with voltage {:?}",
            if positive { "positive" } else { "negative" },
            if enable { "enabled" } else { "disabled" },
            voltage
        );
        Ok(())
    }

    /// Stop ADC1 conversions
    pub fn stop_adc1(&mut self) -> Result<()> {
        self.write_cmd(Command::Stop1)
    }

    /// Stop ADC2 conversions
    pub fn stop_adc2(&mut self) -> Result<()> {
        self.write_cmd(Command::Stop2)
    }

    /// Start ADC1 conversions
    pub fn start_adc1(&mut self) -> Result<()> {
        self.write_cmd(Command::Start1)
    }

    /// Start ADC2 conversions
    pub fn start_adc2(&mut self) -> Result<()> {
        self.write_cmd(Command::Start2)
    }
}
