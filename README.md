# Embedded Rust: Raspberry Pi Pico 2 W

Lab work from the Microprocessor Architecture course at Politehnica University of Bucharest (Erasmus, spring 2025). It's written in `no_std` async Rust with [Embassy](https://embassy.dev) and runs on a Raspberry Pi Pico 2 W (RP2350 chip, CYW43439 Wi-Fi).

## What's in `src/main.rs`

| Exercise | Status | What it does |
|---|---|---|
| Wi-Fi access point | active | Starts an open access point on channel 6 with the static IP `192.168.4.1/24` |
| Wi-Fi client | commented out | Scans for networks, joins an open one and gets an IP address over DHCP |
| Temperature logger | commented out | Reads a BMP280 sensor over I²C, converts the raw value to °C with the compensation formula from the datasheet, saves the last reading to an I²C EEPROM and reads it back on boot |

Only one `main` can be active at a time. To run a different exercise, uncomment it and comment out the others.

## Build

Requires the Rust toolchain and target pinned in `rust-toolchain.toml` (Rust 1.85, `thumbv8m.main-none-eabihf`).

```bash
cargo build --release
```

Flash the resulting ELF to the board with [probe-rs](https://probe.rs).

## Credits

The project skeleton, `embassy-lab-utils`, the linker script and the Wi-Fi firmware come from the course lab template. My code is in `src/main.rs`.
