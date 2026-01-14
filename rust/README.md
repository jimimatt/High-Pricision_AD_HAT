# ADS1263 Rust Driver

A Rust driver for the Texas Instruments ADS1263 32-bit, high-precision delta-sigma ADC, designed for use with the Waveshare High-Precision AD HAT on Raspberry Pi.

## Features

- ✅ Full support for ADC1 (32-bit) and ADC2 (24-bit)
- ✅ Single-ended and differential input modes
- ✅ Configurable gain (1x to 64x for ADC1, 1x to 128x for ADC2)
- ✅ Multiple data rates (2.5 SPS to 38.4 kSPS)
- ✅ Digital filter selection (Sinc1-4, FIR)
- ✅ RTD (Resistance Temperature Detector) measurement support
- ✅ DAC output control for sensor biasing
- ✅ Automatic CRC checksum verification
- ✅ Comprehensive error handling with `thiserror`
- ✅ Detailed logging with `log` crate

## Hardware Requirements

- Raspberry Pi (tested on Pi 5, should work on Pi 3/4)
- [Waveshare High-Precision AD HAT](https://www.waveshare.com/High-Precision-AD-HAT.htm) (or compatible ADS1263 board)
- SPI enabled on the Raspberry Pi

## Pin Connections

| AD HAT | Raspberry Pi (BCM) | Function     |
|--------|-------------------|--------------|
| VCC    | 3.3V              | Power        |
| GND    | GND               | Ground       |
| DIN    | GPIO 10           | SPI MOSI     |
| DOUT   | GPIO 9            | SPI MISO     |
| SCLK   | GPIO 11           | SPI CLK      |
| CS     | GPIO 22           | Chip Select  |
| DRDY   | GPIO 17           | Data Ready   |
| RST    | GPIO 18           | Reset        |
| AVDD   | 5V or 2.5V        | Analog VDD   |
| AVSS   | GND or -2.5V      | Analog VSS   |

## Installation

### Prerequisites

1. Enable SPI on your Raspberry Pi:
   ```bash
   sudo raspi-config
   # Navigate to: Interface Options -> SPI -> Enable
   ```

2. Verify SPI is enabled:
   ```bash
   ls /dev/spidev*
   # Should show: /dev/spidev0.0  /dev/spidev0.1
   ```

3. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Building

```bash
# Clone or copy the project
cd ads1263-rust

# Build in release mode
cargo build --release

# Run (may require root for GPIO/SPI access)
./target/release/ads1263-demo

# Or run with debug logging (may require root for GPIO/SPI access)
RUST_LOG=debug ./target/release/ads1263-demo
```

## Usage

### Basic Example

```rust
use ads1263::{Ads1263, Hal, DataRate, InputMode};

fn main() -> ads1263::Result<()> {
    // Initialize hardware
    let hal = Hal::new()?;
    let mut adc = Ads1263::new(hal);

    // Configure for single-ended measurements
    adc.set_mode(InputMode::SingleEnded);
    adc.init_adc1(DataRate::Sps400)?;

    // Read channel 0
    let raw = adc.get_channel_value(0)?;
    let voltage = Ads1263::raw_to_voltage_adc1(raw, 5.0);

    println!("Channel 0: {:.6} V", voltage);
    Ok(())
}
```

### Reading Multiple Channels

```rust
let channels = [0, 1, 2, 3, 4];
let values = adc.get_all(&channels)?;

for (i, raw) in values.iter().enumerate() {
    let voltage = Ads1263::raw_to_voltage_adc1(*raw, 5.0);
    println!("Channel {}: {:.6} V", channels[i], voltage);
}
```

### Differential Mode

```rust
adc.set_mode(InputMode::Differential);
adc.init_adc1(DataRate::Sps400)?;

// Read differential pair (AIN0 - AIN1)
let raw = adc.get_channel_value(0)?;
let voltage = Ads1263::raw_to_voltage_adc1(raw, 5.0);
```

### RTD Temperature Measurement

```rust
let raw = adc.read_rtd(Delay::Delay8_8ms, Gain::Gain1, DataRate::Sps20)?;

// Calculate resistance (2000Ω reference resistor)
let resistance = Ads1263::rtd_to_resistance(raw, 2000.0);

// Convert to temperature (PT100 sensor)
let temperature = Ads1263::pt100_to_celsius(resistance);
println!("Temperature: {:.2} °C", temperature);
```

### Custom Pin Configuration

```rust
use ads1263::{Hal, PinConfig};

let config = PinConfig {
    rst: 18,   // BCM pin for reset
    cs: 22,    // BCM pin for chip select
    drdy: 17,  // BCM pin for data ready
};

let hal = Hal::with_pins(config)?;
```

## API Reference

### Main Types

| Type | Description |
|------|-------------|
| `Ads1263` | Main ADC driver |
| `Hal` | Hardware abstraction layer |
| `DataRate` | ADC1 sample rate (2.5 to 38400 SPS) |
| `Adc2DataRate` | ADC2 sample rate (10 to 800 SPS) |
| `Gain` | ADC1 PGA gain (1x to 64x) |
| `Adc2Gain` | ADC2 PGA gain (1x to 128x) |
| `InputMode` | Single-ended or differential |
| `Delay` | Conversion delay |
| `DacVoltage` | DAC output voltage |

### Error Handling

```rust
use ads1263::{Ads1263Error, Result};

fn read_adc() -> Result<f64> {
    let hal = Hal::new()?;  // Propagates GPIO/SPI errors
    let mut adc = Ads1263::new(hal);
    
    adc.init_adc1(DataRate::Sps400)?;
    
    let raw = adc.get_channel_value(0)?;  // Propagates timeout/channel errors
    Ok(Ads1263::raw_to_voltage_adc1(raw, 5.0))
}

// Handle specific errors
match read_adc() {
    Ok(v) => println!("Voltage: {}", v),
    Err(Ads1263Error::Timeout) => eprintln!("ADC timeout!"),
    Err(Ads1263Error::InvalidChannel(ch, max)) => {
        eprintln!("Invalid channel {} (max: {})", ch, max)
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Comparison with C Version

| Feature | C Version | Rust Version |
|---------|-----------|--------------|
| Error handling | Return codes | `Result<T, E>` type |
| Memory safety | Manual | Guaranteed by compiler |
| Type safety | Weak (enums are ints) | Strong (real enums) |
| Resource cleanup | Manual `Exit()` call | Automatic via `Drop` |
| Configuration | Compile-time `#ifdef` | Runtime + features |
| Documentation | Comments | Rustdoc + examples |

## Troubleshooting

### "Failed to open SPI device"
- Ensure SPI is enabled: `sudo raspi-config`
- Check device exists: `ls /dev/spidev*`
- Run with root: `sudo ./target/release/ads1263-demo`

### "Timeout waiting for DRDY"
- Check wiring, especially DRDY pin
- Verify the HAT is powered correctly
- Try a slower data rate

### "Invalid chip ID"
- Ensure proper power supply (5V for AVDD)
- Check SPI wiring (MOSI, MISO, SCLK)
- Verify reset pin connection

## License

MIT License - See LICENSE file for details.

## Acknowledgments

- Based on the [Waveshare High-Precision AD HAT](https://www.waveshare.com/wiki/High-Precision_AD_HAT) C library
- Texas Instruments ADS1263 datasheet

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
