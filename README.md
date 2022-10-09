# Neopixel example with Drogue IoT device

Run:

```shell
cd application

probe-rs-cli erase --chip nrf52840_xxAA
#probe-rs-cli download softdevice.hex --format Hex --chip nRF52840_xxAA
#cargo flash --manifest-path ../bootloader/Cargo.toml --release --chip nRF52840_xxAA
DEFMT_LOG=info cargo run --release --no-default-features --features debug
```

Just flash:

```shell
cd application

probe-rs-cli erase --chip nrf52840_xxAA
cargo flash --release --chip nrf52840_xxAA
```