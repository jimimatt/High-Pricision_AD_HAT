//! ADS1263 Demo Application
//!
//! This example demonstrates continuous ADC readings from multiple channels,
//! similar to the original C demo from Waveshare.

use ads1263::{Ads1263, Adc2DataRate, DataRate, Delay, Gain, Hal, InputMode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Reference voltage (modify according to actual voltage)
/// External AVDD and AVSS (Default), or internal 2.5V
const REFERENCE_VOLTAGE: f64 = 5.08;

/// Test modes - set one to true
const TEST_ADC1: bool = true;
const TEST_ADC1_RATE: bool = false;
const TEST_ADC2: bool = false;
const TEST_RTD: bool = false;

fn main() -> ads1263::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("ADS1263 Rust Demo");
    println!("/***********************************/");

    // Setup Ctrl+C handler for graceful exit
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\r\nReceived Ctrl+C, exiting...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Initialize hardware
    let hal = Hal::new()?;
    let mut adc = Ads1263::new(hal);

    // Set single-ended mode (0 = single-ended, 1 = differential)
    adc.set_mode(InputMode::SingleEnded);

    if TEST_ADC1 {
        test_adc1(&mut adc, &running)?;
    } else if TEST_ADC1_RATE {
        test_adc1_rate(&mut adc)?;
    } else if TEST_ADC2 {
        test_adc2(&mut adc, &running)?;
    } else if TEST_RTD {
        test_rtd(&mut adc)?;
    }

    println!("\r\nEND");
    Ok(())
}

/// Test ADC1 - Continuous reading of multiple channels
fn test_adc1(adc: &mut Ads1263, running: &Arc<AtomicBool>) -> ads1263::Result<()> {
    println!("TEST_ADC1");

    // Initialize ADC1 at 400 SPS
    // The faster the rate, the worse the stability
    // Choose a suitable digital filter in REG_MODE1
    adc.init_adc1(DataRate::Sps400)?;

    // Define channels to read (must be less than 10)
    const CHANNEL_COUNT: usize = 5;
    let channels: [u8; CHANNEL_COUNT] = [0, 1, 2, 3, 4];

    while running.load(Ordering::SeqCst) {
        // Read all channels
        let values = adc.get_all(&channels)?;

        // Print values
        for (i, &raw) in values.iter().enumerate() {
            let voltage = Ads1263::raw_to_voltage_adc1(raw, REFERENCE_VOLTAGE);
            if voltage < 0.0 {
                println!("IN{} is {:.6} V", channels[i], voltage);
            } else {
                println!("IN{} is  {:.6} V", channels[i], voltage);
            }
        }

        // Move cursor up to overwrite previous output (like the C version)
        for _ in 0..CHANNEL_COUNT {
            print!("\x1B[1A"); // Move cursor up one line
        }
    }

    Ok(())
}

/// Test ADC1 sample rate
fn test_adc1_rate(adc: &mut Ads1263) -> ads1263::Result<()> {
    println!("TEST_ADC1_RATE");

    adc.init_adc1(DataRate::Sps400)?;

    let is_single_channel = true;
    let iterations = 10000;

    let start = std::time::Instant::now();

    if is_single_channel {
        for _ in 0..iterations {
            let _ = adc.get_channel_value(0)?;
        }

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;
        println!("{:.2} ms", time_ms);
        println!("Single channel: {:.2} kHz", iterations as f64 / time_ms);
    } else {
        for _ in 0..iterations {
            let _ = adc.get_channel_value(0)?;
        }

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;
        println!("{:.2} ms", time_ms);
        println!("Multi channel: {:.2} kHz", iterations as f64 / time_ms);
    }

    Ok(())
}

/// Test ADC2 - Read all 10 channels from the 24-bit auxiliary ADC
fn test_adc2(adc: &mut Ads1263, running: &Arc<AtomicBool>) -> ads1263::Result<()> {
    println!("TEST_ADC2");

    // Initialize ADC2 at 100 SPS
    adc.init_adc2(Adc2DataRate::Sps100)?;

    while running.load(Ordering::SeqCst) {
        // Read all 10 channels from ADC2
        let values = adc.get_all_adc2()?;

        // Print values
        for (i, &raw) in values.iter().enumerate() {
            let voltage = Ads1263::raw_to_voltage_adc2(raw, REFERENCE_VOLTAGE);
            if voltage < 0.0 {
                println!("IN{} is {:.6} V", i, voltage);
            } else {
                println!("IN{} is  {:.6} V", i, voltage);
            }
        }

        // Move cursor up 10 lines
        print!("\x1B[10A");
    }

    Ok(())
}

/// Test RTD (Resistance Temperature Detector)
fn test_rtd(adc: &mut Ads1263) -> ads1263::Result<()> {
    println!("TEST_RTD");

    adc.init_adc1(DataRate::Sps20)?;

    let raw = adc.read_rtd(Delay::Delay8_8ms, Gain::Gain1, DataRate::Sps20)?;

    // Calculate resistance
    // 2000.0 = 2000Ω reference resistor
    // 2.0 = 2 × IDAC current
    let resistance = Ads1263::rtd_to_resistance(raw, 2000.0);
    println!("Resistance: {:.2} Ω", resistance);

    // Calculate temperature using PT100 coefficient (α = 0.00385)
    let temperature = Ads1263::pt100_to_celsius(resistance);
    println!("Temperature: {:.2} °C", temperature);

    Ok(())
}
