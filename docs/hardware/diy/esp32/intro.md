# **Doggie ESP32**


## **Description**

This implementation provides the hardware abstraction for **Doggie** and **evilDoggie** to run in the ESP32 microcontrollers family.

## **Supported Configurations**


### Doggie
The ESP32 implementation supports the following configurations.

1. **USB, UART0 or BLE, and MCP2515 (SPI to CAN)**
2. **USB, UART0 or BLE, and TWAI (Internal controller)**

For a detailed documentation refer to the [Doggie on ESP32 documentation](./doggie.md)

### evilDoggie
The ESP32 implementation supports only one configuration.

For a detailed documentation refer to the [evilDoggie on ESP32 documentation](./evil_doggie.md)

## **Notes on debugging**

The UART-USB or SerialJtagUsb bridge of the esp32 is usually used to flash, write logs and debugging, but as we will be using it as a serial interface for CAN Bus, we need another way to log and debug. For that we set up another UART interface that will print logs. In the "Connections Variants" table we could find the corresponding UART TX pins as **LOGS**. For the ESP32 the GPIO is `D10` and for the ESP32C3 the `3`.

### How to see logs

First we need to connect the UART TX GPIO to a UART-USB bridge.

```
$ ./monitor.sh { LOGs UART } { ELF FILE }
```

 Suppose the bridge has the /dev/ttyUSB0 interface and we want the logs from evilDoggie on ESP32 (xtensa), we will run the following command:

```
$ ./monitor.sh /dev/ttyUSB0 target/xtensa-esp32-none-elf/release/evil_doggie
```


## **How to Flash a Release**
1. Install `espflash`
```
  $ cargo install espflash
  $ cargo install cargo-espflash
```

2. Download the release `doggie_esp32`.
3. Run `espflash flash --monitor -L defmt doggie_esp32`

## **How to Compile and Flash**

### **Prerequisites**

1. Install **Rust** and **cargo**.
   Follow the installation instructions from the official [Rust website](https://www.rust-lang.org/tools/install).


2. Install `ldproxy`, `espup` and the ESP32 toolchain:
```
  $ cargo install ldproxy
  $ cargo install espup --version 0.13.0
  $ espup install --toolchain-version 1.84.0
  $ . $HOME/export-esp.sh           # Or add to .zshrc/.bashrc
```

3. Install `espflash`
```
  $ cargo install cargo-espflash --version 3.2.0
```

### **Compile and Flash the Firmware**

1. First we need to connect the ESP32 to the PC via USB.

2. Then we need to select the desired project with the `--bin` flag:

    * `doggie`
    * `evil_doggie` 

3. We also need to select the feature for the specific board, for now only two are supported:

    * `esp32`
    * `esp32c3`

4. And then we flash with:
```
DEFMT_LOG=off cargo {BOARD} --bin doggie --disable-default-features --features {FEATURES}
```