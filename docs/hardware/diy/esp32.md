# **Doggie ESP32**


## **Description**
This implementation provides a **CAN Bus to USB or BLE adapter** using the **ESP32** microcontroller (commonly available in devkits such as the ESP-WROOM-32 we will use as example). The adapter uses the **slcan protocol** (CAN over Serial), making it compatible with popular software tools such as **SocketCAN**, **Python-can**, and other slcan-compatible applications. Since these boards include a UART bridge connected to the UART0 peripheral, SLCAN will be available through UART0 and USB using the default configuration.

---

## **Supported Configurations**

The ESP32 implementation supports the following configurations.

As the ESP32 doesn't have 5v tolerant GPIOs, we should modify the MCP2515 or use a logic level shifter in order to make it compatible. Read [MCP2515 module compatibility note](mcp.md) for more information.

1. **USB, UART0 or BLE, and MCP2515 (SPI to CAN)**
    - The **USB** port, the **UART0** port and the BLE of the ESP32 can be used for communication with the host system.
    - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.
    - This configuration allows the device to interface with a CAN network while communicating with the host via USB or BLE.

    __Connections__ (MCP2515 mod):
    
    | Function |    MCP2515     |
    | :------: | :------------: |
    |   Vcc 3.3|       VCC      |
    |   Vcc 5v | Tranceiver Vcc |
    |   GND    |       GND      |
    |   MOSI   |       SI       |
    |   MISO   |       SO       |
    |   Clock  |       SCK      |
    |   CS     |       CS       |

    ![alt text](../../res/esp32_mcp_mod.png)

    __Connections__ (MCP2515 with level shifter):

    | Function | Level Shifter | MCP2515 |
    | :------: | :-----------: | :-----: |
    |   Vcc    |        -      |    5v   |
    |   GND    |        -      |    GND  |
    |   MOSI   | <-----------> |    SI   |
    |   MISO   | <-----------> |    SO   |
    |   Clock  | <-----------> |    SCK  |
    |   CS     | <-----------> |    CS   |

    ![alt text](../../res/esp32_mcp_ls.png)


2. **USB, UART0 or BLE, and TWAI (Internal controller)**
    - The **USB** port, the **UART0** port and the BLE of the ESP32 can be used for communication with the host system.
    - The TWAI controller is used for CAN Bus communication.
    - This configuration allows the device to interface with a CAN network using only a transceiver while communicating with the host via USB or BLE.

    ![alt text](../../res/esp32_twai.png)

    __Connections__:
    
    | Function |   Tranceiver   |
    | :------: |:-------------: |
    |   Vcc    |       VCC      |
    |   GND    |       GND      |
    |   CAN TX |       TX       |
    |   CAN RX |       RX       |


    For each esp32 varian we will use different pins that are defined but they could be easily changed in the code. Some variants are not implemented but are compatible and will be implemented on demand.

    __Connections variants__:
  
    | Function    |   ESP32  | ESP32c3  |
    | :---------: | :------: | :------: |
    |    Vcc 3.3  |   3v3    |    3.3   |
    |    Vcc 5v   |  VIN/5v  |    5v    |
    |    GND      |   GND    |     G    |
    |    MOSI     |   D13    |     6    |
    |    MISO     |   D12    |     5    |
    |    Clock    |   D14    |     9    |
    |    CS       |   D15    |     7    |
    |    CAN TX   |   D4     |     10   |
    |    CAN RX   |   D3     |     9    |
    | LOGS (UART) |   D10    |     3    |

---

## **Notes on debugging**

The UART-USB or SerialJtagUsb bridge of the esp32 is usually used to flash, write logs and debugging, but as we will be using it as a serial interface for CAN Bus, we need another way to log and debug. For that we set up another UART interface that will print logs. In the "Connections Variants" table we could find the corresponding UART TX pins as **LOGS**.


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

In order to manage the compilation with different hardware variants we use features. The most important feature is the one that select the board (**esp32**, **esp32c3**, etc). And we have the other features:
* `twai`: Enable the internal CAN controller (TWAI)
* `mcp`: Enable the MCP2515 SPI interface as CAN controller
* `ble`: Enable BLE interface

By default `twai` and `ble` are enabled.

1. Connect ESP32 to the PC via USB

3. Build and flash:
```
DEFMT_LOG=off cargo {BOARD} --bin {BINARY} --disable-default-features --features {FEATURES}
```

For example:
```
# ESP32c3 with TWAI and BLE
DEFMT_LOG=off cargo esp32c3 --bin doggie

# ESP32 with MCP2515 and BLE
DEFMT_LOG=off cargo esp32 --bin doggie --disable-default-features --features mcp,ble
```
