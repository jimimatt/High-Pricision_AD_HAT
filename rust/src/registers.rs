//! ADS1263 Register definitions, commands, and configuration enums
//!
//! This module contains all the register addresses, command codes,
//! and configuration options for the ADS1263 ADC.

#![allow(dead_code)]

/// ADS1263 Register addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    /// Device ID register
    Id = 0x00,
    /// Power control register
    Power = 0x01,
    /// Interface configuration register
    Interface = 0x02,
    /// Mode 0 register (delay, chop, etc.)
    Mode0 = 0x03,
    /// Mode 1 register (filter settings)
    Mode1 = 0x04,
    /// Mode 2 register (gain, data rate)
    Mode2 = 0x05,
    /// Input multiplexer register
    InpMux = 0x06,
    /// Offset calibration byte 0
    OfCal0 = 0x07,
    /// Offset calibration byte 1
    OfCal1 = 0x08,
    /// Offset calibration byte 2
    OfCal2 = 0x09,
    /// Full-scale calibration byte 0
    FsCal0 = 0x0A,
    /// Full-scale calibration byte 1
    FsCal1 = 0x0B,
    /// Full-scale calibration byte 2
    FsCal2 = 0x0C,
    /// IDAC multiplexer register
    IdacMux = 0x0D,
    /// IDAC magnitude register
    IdacMag = 0x0E,
    /// Reference multiplexer register
    RefMux = 0x0F,
    /// Positive DAC register
    TdacP = 0x10,
    /// Negative DAC register
    TdacN = 0x11,
    /// GPIO connection register
    GpioCon = 0x12,
    /// GPIO direction register
    GpioDir = 0x13,
    /// GPIO data register
    GpioDat = 0x14,
    /// ADC2 configuration register
    Adc2Cfg = 0x15,
    /// ADC2 input multiplexer register
    Adc2Mux = 0x16,
    /// ADC2 offset calibration byte 0
    Adc2Ofc0 = 0x17,
    /// ADC2 offset calibration byte 1
    Adc2Ofc1 = 0x18,
    /// ADC2 full-scale calibration byte 0
    Adc2Fsc0 = 0x19,
    /// ADC2 full-scale calibration byte 1
    Adc2Fsc1 = 0x1A,
}

/// ADS1263 Command codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// Reset the ADC
    Reset = 0x06,
    /// Start ADC1 conversions
    Start1 = 0x08,
    /// Stop ADC1 conversions
    Stop1 = 0x0A,
    /// Start ADC2 conversions
    Start2 = 0x0C,
    /// Stop ADC2 conversions
    Stop2 = 0x0E,
    /// Read ADC1 data
    RData1 = 0x12,
    /// Read ADC2 data
    RData2 = 0x14,
    /// ADC1 system offset calibration
    SysOCal1 = 0x16,
    /// ADC1 system gain calibration
    SysGCal1 = 0x17,
    /// ADC1 self offset calibration
    SelfOCal1 = 0x19,
    /// ADC2 system offset calibration
    SysOCal2 = 0x1B,
    /// ADC2 system gain calibration
    SysGCal2 = 0x1C,
    /// ADC2 self offset calibration
    SelfOCal2 = 0x1E,
    /// Read register command base
    RReg = 0x20,
    /// Write register command base
    WReg = 0x40,
}

/// ADC1 Programmable Gain Amplifier settings
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Gain {
    /// Gain = 1 (default)
    #[default]
    Gain1 = 0,
    /// Gain = 2
    Gain2 = 1,
    /// Gain = 4
    Gain4 = 2,
    /// Gain = 8
    Gain8 = 3,
    /// Gain = 16
    Gain16 = 4,
    /// Gain = 32
    Gain32 = 5,
    /// Gain = 64
    Gain64 = 6,
}

/// ADC1 Data rate settings (samples per second)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DataRate {
    /// 2.5 SPS
    Sps2_5 = 0,
    /// 5 SPS
    Sps5 = 1,
    /// 10 SPS
    Sps10 = 2,
    /// 16.6 SPS
    Sps16_6 = 3,
    /// 20 SPS
    Sps20 = 4,
    /// 50 SPS
    Sps50 = 5,
    /// 60 SPS
    Sps60 = 6,
    /// 100 SPS
    Sps100 = 7,
    /// 400 SPS (default)
    #[default]
    Sps400 = 8,
    /// 1200 SPS
    Sps1200 = 9,
    /// 2400 SPS
    Sps2400 = 10,
    /// 4800 SPS
    Sps4800 = 11,
    /// 7200 SPS
    Sps7200 = 12,
    /// 14400 SPS
    Sps14400 = 13,
    /// 19200 SPS
    Sps19200 = 14,
    /// 38400 SPS
    Sps38400 = 15,
}

/// ADC1 Conversion delay settings
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Delay {
    /// No delay
    Delay0 = 0,
    /// 8.7 µs delay
    Delay8_7us = 1,
    /// 17 µs delay
    Delay17us = 2,
    /// 35 µs delay (default)
    #[default]
    Delay35us = 3,
    /// 169 µs delay
    Delay169us = 4,
    /// 139 µs delay
    Delay139us = 5,
    /// 278 µs delay
    Delay278us = 6,
    /// 555 µs delay
    Delay555us = 7,
    /// 1.1 ms delay
    Delay1_1ms = 8,
    /// 2.2 ms delay
    Delay2_2ms = 9,
    /// 4.4 ms delay
    Delay4_4ms = 10,
    /// 8.8 ms delay
    Delay8_8ms = 11,
}

/// ADC2 Programmable Gain Amplifier settings
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Adc2Gain {
    /// Gain = 1 (default)
    #[default]
    Gain1 = 0,
    /// Gain = 2
    Gain2 = 1,
    /// Gain = 4
    Gain4 = 2,
    /// Gain = 8
    Gain8 = 3,
    /// Gain = 16
    Gain16 = 4,
    /// Gain = 32
    Gain32 = 5,
    /// Gain = 64
    Gain64 = 6,
    /// Gain = 128
    Gain128 = 7,
}

/// ADC2 Data rate settings (samples per second)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Adc2DataRate {
    /// 10 SPS
    Sps10 = 0,
    /// 100 SPS (default)
    #[default]
    Sps100 = 1,
    /// 400 SPS
    Sps400 = 2,
    /// 800 SPS
    Sps800 = 3,
}

/// DAC output voltage settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DacVoltage {
    /// 4.5V output
    Volt4_5 = 0b01001,
    /// 3.5V output
    Volt3_5 = 0b01000,
    /// 3.0V output
    Volt3_0 = 0b00111,
    /// 2.75V output
    Volt2_75 = 0b00110,
    /// 2.625V output
    Volt2_625 = 0b00101,
    /// 2.5625V output
    Volt2_5625 = 0b00100,
    /// 2.53125V output
    Volt2_53125 = 0b00011,
    /// 2.515625V output
    Volt2_515625 = 0b00010,
    /// 2.5078125V output
    Volt2_5078125 = 0b00001,
    /// 2.5V output (midpoint)
    Volt2_5 = 0b00000,
    /// 2.4921875V output
    Volt2_4921875 = 0b10001,
    /// 2.484375V output
    Volt2_484375 = 0b10010,
    /// 2.46875V output
    Volt2_46875 = 0b10011,
    /// 2.4375V output
    Volt2_4375 = 0b10100,
    /// 2.375V output
    Volt2_375 = 0b10101,
    /// 2.25V output
    Volt2_25 = 0b10110,
    /// 2.0V output
    Volt2_0 = 0b10111,
    /// 1.5V output
    Volt1_5 = 0b11000,
    /// 0.5V output
    Volt0_5 = 0b11001,
}

/// Input mode selection for ADC channels
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputMode {
    /// Single-ended input mode (10 channels: AIN0-AIN9 vs AINCOM)
    #[default]
    SingleEnded,
    /// Differential input mode (5 channel pairs: AIN0-AIN1, AIN2-AIN3, etc.)
    Differential,
}

/// Digital filter selection for ADC1
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DigitalFilter {
    /// Sinc1 filter
    Sinc1 = 0x04,
    /// Sinc2 filter
    Sinc2 = 0x24,
    /// Sinc3 filter
    Sinc3 = 0x44,
    /// Sinc4 filter
    Sinc4 = 0x64,
    /// FIR filter (default, best for 50/60Hz rejection)
    #[default]
    Fir = 0x84,
}

/// Reference voltage source selection
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ReferenceSource {
    /// Internal 2.5V reference
    Internal2_5V = 0x00,
    /// External reference on AIN0/AIN1
    ExternalAin01 = 0x09,
    /// External reference on AIN2/AIN3
    ExternalAin23 = 0x12,
    /// External reference on AIN4/AIN5
    ExternalAin45 = 0x1B,
    /// AVDD and AVSS as reference (default)
    #[default]
    AvddAvss = 0x24,
}
