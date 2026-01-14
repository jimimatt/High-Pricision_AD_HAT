# High-Precision AD HAT (C Library)

## Supported Platforms

- Raspberry Pi 4B and earlier (using bcm2835 library or sysfs fallback)
- Raspberry Pi 5 (automatic sysfs/spidev with GPIO offset detection)
- NVIDIA Jetson

See [`examples/main.c`](examples/main.c) for the detailed test routine.

## Pin Connections

Pin connections can also be viewed in [`lib/Config/DEV_Config.c`](lib/Config/DEV_Config.c).

| AD HAT | RPI (BCM) | Notes |
|--------|-----------|-------|
| VCC    | -         | Other devices can connect directly to 3.3V |
| GND    | GND       | |
| DIN    | 10        | MOSI |
| DOUT   | 9         | MISO |
| SCLK   | 11        | SCLK |
| CS     | 22        | |
| DRDY   | 17        | |
| REST   | 18        | |
| AVDD   | -         | 5V or 2.5V |
| AVSS   | -         | GND or -2.5V |

## Install Libraries

### BCM2835 (Pi 4 and earlier only - not needed for Pi 5)

```bash
wget http://www.airspayce.com/mikem/bcm2835/bcm2835-1.68.tar.gz
tar zxvf bcm2835-1.68.tar.gz
cd bcm2835-1.68/
sudo ./configure && sudo make && sudo make check && sudo make install
```

> **Note:** On Raspberry Pi 5, the bcm2835 library is not supported (`/dev/gpiomem` does not exist). The program automatically falls back to sysfs/spidev and detects the Pi 5 GPIO offset (571) at runtime.

## Basic Use

The factory hardware default COM has been connected to GND. The program has configured IN0 and IN1 as two analog inputs. You can connect IN0 or IN1 and GND to measure the target voltage.

### Raspberry Pi

```bash
make clean
make
sudo ./main
```

### NVIDIA Jetson

```bash
make clean
make
sudo ./main
```

For more information, visit the [official Waveshare Wiki](https://www.waveshare.net/wiki/High-Precision_AD_HAT).

---

## Raspberry Pi Notes (Pi 4 / Pi 5)

### Build on-device

```bash
sudo apt update
sudo apt install build-essential
# Install bcm2835 library (Pi 4 and earlier only, see above)
# Enable SPI via raspi-config (Interface Options -> SPI)
make
sudo ./main
```

### Automatic Fallback

If `bcm2835_init()` fails (e.g., `/dev/gpiomem` unavailable on Pi 5), the program will fall back to the kernel spidev + sysfs GPIO path automatically. On Pi 5, the GPIO offset (571) is auto-detected. Runtime diagnostics will show which backend is in use and the configured GPIO pin numbers.

### Force sysfs/spidev Backend

To force the sysfs/spidev backend at build time:

```bash
make USELIB_RPI=USE_DEV_LIB
```

### Troubleshooting

- Ensure SPI is enabled and `/dev/spidev0.0` exists
- Ensure your user has access to `spi` and `gpio` groups, or run as root
- Check runtime diagnostics printed by the binary to see the selected backend and configured pins
- On Pi 5, you should see:
  ```
  Raspberry Pi 5 detected: using GPIO offset 571
  GPIO via sysfs: RST=589 CS=593 DRDY=588
  ```
