Final Structure:
```
- Introduction: intro.md
- Doggie:
  - Introduction: doggie/intro.md
  - Get Started: doggie/get_started.md
- evilDoggie:
  - Introduction: evil_doggie/intro.md
  - Get Started: evil_doggie/get_started.md
- Bluetooth LE: ble.md
- slcan: slcan.md
- Hardware:
  - Faraday: hardware/faraday.md
  - DIY:
    - Introduction: hardware/diy/intro.md
    - Microcontrollers:
      - ESP32: hardware/diy/esp32.md
      - RP2040 (Raspberry Pico): hardware/diy/rp.md
      - STM32F1 (bluepill): hardware/diy/bluepill.md
    - MCP2515 Notes: hardware/mcp.md
- evilDoggie lab:
  - Introduction: workshop/intro.md
```

### intro.md
# Doggie and evilDoggie Project Overview

Doggie is an open-source, modular project designed to build a DIY CAN Bus to serial adapter (USB, BLE, UART). It connects your computer to a CAN Bus network using the slcan protocol (CAN over Serial) for compatibility with tools like SocketCAN and Python-can. The project emphasizes modularity, supporting various microcontrollers (e.g., RP2040, STM32F103C8, ESP32) and CAN controllers (built-in or MCP2515).

evilDoggie is the offensive firmware variant of Doggie, tailored for automotive security research and low-level CAN Bus manipulation. It enables advanced attacks like spoofing, bus off, double receive, and physical-layer overrides, making it ideal for red teaming and vulnerability testing in simulated or real CAN environments.

Developed by Faraday Security, the project was presented at Black Hat Arsenal on August 6-7, 2025, in Las Vegas. Both variants are open-source under the MIT License and available on GitHub at [https://github.com/infobyte/doggie](https://github.com/infobyte/doggie). Use responsibly for research and training only.

For detailed introductions, see the sub-sections for Doggie and evilDoggie.

### doggie/intro.md
# Doggie Introduction

**Doggie** is an open-source, modular project designed to build a DIY CAN Bus to serial adapter (USB, BLE, UART). The device connects your computer to a CAN Bus network and uses the **slcan protocol** (CAN over Serial) to ensure compatibility with popular tools like **SocketCAN**, **Python-can**, and other slcan-compatible software.  

The project emphasizes **modularity**, allowing users to select from various hardware configurations with different microcontrollers and CAN transceivers, making it accessible and cost-effective. Whether you're using a microcontroller's built-in CAN controller or an **MCP2515** (SPI to CAN) module, **Doggie** adapts to your needs.

## Supported Configurations  

### Microcontrollers:
- **Raspberry Pi Pico (RP2040)**:  [doggie_pico](./doggie_pico/README.md)
- **STM32F103C8 (Bluepill)**: [doggie_bluepill](./doggie_bluepill/README.md)
- **ESP32**: [doggie_esp32](./doggie_esp32/README.md)

### CAN Controllers:  
- Built-in CAN controllers (if supported by the microcontroller)  
- **MCP2515** (SPI to CAN, see [compatibility modification](../docs/mcp_mod.md))  

### Serial Connectivity:
- **Microcontroller USB** (native USB support)
- **UART with USB Bridge**
- **Bluetooth**  

Each hardware configuration is detailed in its respective subdirectory under the `doggie_{bsp}` folder.

## Disclaimer  
This project is a **work in progress**, and contributions are highly encouraged! While it is functional, some features may still be under development.  

If you encounter issues or have suggestions for improvements, please feel free to open an issue or submit a pull request.  

### License  
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

### doggie/get_started.md
# Doggie Get Started

## Prerequisites

If you want to build the project, you will need Rust and Cargo.
Follow the installation instructions from the official [Rust website](https://doc.rust-lang.org/book/ch01-01-installation.html).

The instructions of how to build and flash Doggie are in the README.md of each possible configuration, as
it depends on the microcontroller. For more information check `doggie_{bsp}/README.md`.

## Using SocketCAN on Linux  

SocketCAN is a powerful framework for interfacing with CAN networks. Once the device is connected, follow these steps:

### 1. Install CAN Utilities  
```bash
# On Ubuntu
sudo apt-get install can-utils
```

### 2. Attach the Device  
Identify the device (e.g., `/dev/ttyUSB0`) and attach it.

First we start the slcan daemon with the configuration:
* The `-sX` argument determines the speed:
  - s0: 10 kbit/s
  - s1: 20 kbit/s
  - s2: 50 kbit/s
  - s3: 100 kbit/s
  - s4: 125 kbit/s
  - s5: 250 kbit/s
  - s6: 500 kbit/s
  - s7: 800 kbit/s
  - s8: 1 Mbit/s
* The `-S{baudrate}` determines the serial interface baudrate (Not necessary on most USB implementations)

```bash
# Start the slcan daemon:
sudo slcand -s5 -S115200 /dev/ttyUSB0 can0

# Set the interface UP
sudo ifconfig can0 up
```

### 3. Send/Receive CAN Messages  
- **Send a CAN message:**  
  ```bash
  cansend can0 123#11223344
  ```

- **Receive CAN messages:**  
  ```bash
  candump can0
  ```

For more advanced commands, refer to the [SocketCAN documentation](https://www.kernel.org/doc/Documentation/networking/can.txt).

## BLE  

As some boards supports BLE to send and receive serial information we need a way to bridge the BLE data to a serial interface implementing the NUS service.
Visit the [BLE notes](/docs/bluetooth_notes.md) for mor information.

In linux we could use the `ble-serial` package as we show:

First we install the package
```
$ pip install ble-serial
```

And then we run the bridge with the mac address of our device
```
$ ble-serial -d 12:00:3B:01:B2:A5 --write-with-response
10:38:57.733 | INFO | linux_pty.py: Port endpoint created on /tmp/ttyBLE -> /dev/pts/4
10:38:57.733 | INFO | ble_client.py: Receiver set up
10:38:57.938 | INFO | ble_client.py: Trying to connect with 12:00:3B:01:B2:A5: esp32c3
10:38:59.627 | INFO | ble_client.py: Device 12:00:3B:01:B2:A5 connected
10:38:59.628 | INFO | ble_client.py: Found write characteristic 6e400002-b5a3-f393-e0a9-e50e24dcca9e (H. 2)
10:38:59.628 | INFO | ble_client.py: Found notify characteristic 6e400003-b5a3-f393-e0a9-e50e24dcca9e (H. 4)
10:38:59.699 | INFO | main.py: Running main loop!
```

We could see in the logs that the program creates a virtual interface in `/dev/pts/4`, that's the interface we will use as a serial can.

### evil_doggie/intro.md
# evilDoggie Introduction

evilDoggie is the offensive firmware variant of Doggie, created for advanced security research and low-level CAN Bus manipulation. It extends Doggie's capabilities with features for exploiting CAN protocol weaknesses, such as:

- Spoofing messages with bit-level precision.
- Injecting errors to trigger double receives or bus off states.
- Forcing dominant bits to override recessive ones at the physical layer.
- Bus takeover to silence other ECUs.
- Custom attack scripting for targeted vulnerabilities.

These features make evilDoggie a powerful tool for deeper research into how CAN networks can be broken and secured.

⚠ Important: evilDoggie is for research and training purposes only. Use it responsibly!

Project repository: [https://github.com/infobyte/doggie](https://github.com/infobyte/doggie)

### evil_doggie/get_started.md
# evilDoggie Get Started

## Evil Mode

evilDoggie has a different approach; it uses the serial interface to display a menu that lets the user configure attacks. The --omap lfcrlf and --imap lfcrlf arguments are used to match the control characters of the evilDoggie’s terminal. To exit from picocom, the keybinding is [ctrl + a, ctrl + x].

Now you should be in the evilDoggie console! If you press a key, you will see a banner and a prompt with ‘>’. You can type ‘help’ to see the available commands.

"GOOD" and "EVIL" is used to select the mode in which the board will boot. GOOD for Doggie and EVIL for evilDoggie. Note that you will need to reboot the board after switching modes if it is already powered.

## The Logic Behind evilDoggie Attacks

As an attacker you will connect Doggie or evilDoggie to the bus to complete the challenges.

## How It Is Physically Connected

As the attacks covered by evilDoggie target physical bus characteristics, there needs to be a real CAN bus where an attacker can connect. You can simulate a full CAN Bus without using a Doggie for each ECU! And then, as an attacker, you can connect your own evilDoggie and make an online attack, just like on a real car.

### ble.md
# (Evil)Doggie over Bluetooth

In this note we will cover how and why we implemented BLE for (Evil)Doggie instead of using other Bluetooth solutions like `Rfcomm` or `L2CAP CoC`.  This doesn't mean that future implementations of Bluetooth can't be done by the community, and we encourage the reader to try it. For time restrictions we had to focus only on one implementation.

First of all, I would like to remark something: Doggie was made with one idea in mind, to be as  flexible as we could, speaking in terms of use and compatibility. With that in mind, we chose `slcan` (serial CAN Bus) protocol, as it is the most compatible driver for CAN.

Having said this, the decision of using BLE NUS (Nordic-Uart Service) over a GATT server is clear: to be widely compatible with more hardware. Lets explain why.

## Classic Bluetooth vs BLE
We use `slcan` that, as the name suggests, is a serial implementation of CAN. So the most straight forward incorporation of Bluetooth should be `Rfcomm` as it provides a serial communication over Bluetooth. But `Rfcomm` only supports Classic Bluetooth and not Bluetooth Low Energy. The problem is the compatibility with the microcontrollers we chose for Doggie. Using as an example the ESP32 and its variants, we can see that the BLE is supported for more microcontrollers than the Classic Bluetooth. This decision sacrifices performance over hardware compatibility.

| Device                | Classic Bluetooth | BLE |
| --------------------- | ----------------- | --- |
| ESP32                 | Yes               | Yes |
| ESP32-S2              | No                | No  |
| ESP32-S3              | No                | Yes |
| ESP32-C2              | No                | Yes |
| ESP32-C3              | No                | Yes |
| ESP32-C6              | No                | Yes |
| ESP32-H2              | No                | Yes |
| ESP32-C5              | No                | Yes |
| ESP32-P4              | No                | No  |

## L2CAP CoC vs NUS

After selecting BLE we had another decision to make, as there are two available options. The first one is using the lower layer of the BLE Host, L2CAP. This protocol can be used as a Channel-Oriented connection, as it provides a channel with two ends for communication between devices. Using L2CAP allows to avoid all the upper layers of the stack reducing the overhead in communication. But, the problem with L2CAP is the lack of compatibility with existent tools, specially on Windows. Hence, we chose the other option: NUS.

![alt text](ble_stack.png)

[NUS](https://docs.nordicsemi.com/bundle/ncs-latest/page/nrf/libraries/bluetooth/services/nus.html) (Nordic-Uart Service) is a GATT Service used to receive and write data serving as bridge for UART interfaces. And as we need a serial interface, it fits. Using NUS sacrifices performance and connection stability but allows the use of existing tools even on Windows, increasing the compatibility. The service presents the following characteristics:
### Service UUID

The 128-bit vendor-specific service UUID is `6E400001-B5A3-F393-E0A9-E50E24DCCA9E` (16-bit offset: `0x0001`).

### Characteristics

This service has two characteristics.

#### RX Characteristic (`6E400002-B5A3-F393-E0A9-E50E24DCCA9E`)

Write or Write Without Response

	Write data to the RX Characteristic to send it to the UART interface.

#### TX Characteristic (`6E400003-B5A3-F393-E0A9-E50E24DCCA9E`)

Notify

	Enable notifications for the TX Characteristic to receive data from the application. The application transmits all data that is received over UART as notifications.

## Implementation

The BLE implementation is in `/doggie_ble` and is used as a separate crate for the hardware variants that support BLE. It depends on the `trouble-host` crate, witch is based on `bt_hci`.

Just as an extra, we can see the GATT Server definition:

```Rust
#[gatt_server]
struct Server {
    nus_service: NordicUartService,
}

/// Nordic UART Service
#[gatt_service(uuid = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")]
struct NordicUartService {
    /// TX Characteristic - used to send data to central (notify)
    #[characteristic(uuid = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E", notify, read)]
    tx: Vec<u8, MAX_CMD_LEN>, // Maximum BLE packet size for data

    /// RX Characteristic - used to receive data from central (write)
    #[characteristic(
        uuid = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E",
        write,
        write_without_response
    )]
    rx: Vec<u8, MAX_CMD_LEN>,
}
```

 We made two structs and a function called BleSerial, BleServer and create_ble_pipe. This makes it easier to port BLE to other hardware variants. The only requirement is to implement a Controller, run the BLE server on a separate task and then just use the BleSerial as a serial interface.

```rust
let connector = BleConnector::new(init, peripherals.BT);
let controller = ExternalController::new(connector);

let (ble_server, ble_serial) = create_ble_pipe();

let _ = select(
	ble_server.run(controller),
	async {
		let mut buffer = [0; 255];
		loop {
			let size = ble_serial.read(&mut buffer).await.unwrap();
			ble_serial.write_all(&buffer[0..size]).await;
		}
	}
).await;
```

## Usage

The implementation is compatible with a lot of different UART-BLE programs for different platforms.  For PC, we recomend [ble-serial](https://pypi.org/project/ble-serial), a Python module that allows to create a virtual port bridged with BLE.

To use it, first we install the package

```
$ pip install ble-serial
```


Then you should be able to discover a device with name `Doggie BLE`:

```
$ ble-scan
Started general BLE scan

28:CD:C1:04:D7:E3 (rssi=-62): Doggie BLE
```

And finally, create the bridge using the mac address of the Doggie BLE device

```
$ ble-serial -d 12:00:3B:01:B2:A5 -t 10000
10:38:57.733 | INFO | linux_pty.py: Port endpoint created on /tmp/ttyBLE -> /dev/pts/4
10:38:57.733 | INFO | ble_client.py: Receiver set up
10:38:57.938 | INFO | ble_client.py: Trying to connect with 12:00:3B:01:B2:A5: esp32c3
10:38:59.627 | INFO | ble_client.py: Device 12:00:3B:01:B2:A5 connected
10:38:59.628 | INFO | ble_client.py: Found write characteristic 6e400002-b5a3-f393-e0a9-e50e24dcca9e (H. 2)
10:38:59.628 | INFO | ble_client.py: Found notify characteristic 6e400003-b5a3-f393-e0a9-e50e24dcca9e (H. 4)
10:38:59.699 | INFO | main.py: Running main loop!
```

You should see in the logs that the program creates a virtual interface (in this case `/dev/pts/4`), you can now use that interface as a serial CAN.

### slcan.md
# SLCAN Protocol

SLCAN (Serial Line CAN) is a text-based ASCII protocol for transmitting CAN (Controller Area Network) frames over a serial interface. It is based on the Lawicel SLCAN protocol and is widely used for interfacing CAN networks with computers via USB-to-serial adapters or similar devices. SLCAN enables compatibility with tools like SocketCAN on Linux and Python-can libraries, allowing for easy sending and receiving of CAN messages.

## Key Features
- **ASCII-Based**: Commands and data are sent as human-readable text, making it simple to implement and debug.
- **Compatible Interfaces**: Works with slcan-compatible hardware, including LAWICEL adapters, and supports both local and remote serial ports (usable on Windows, Linux, etc.).
- **Commands**: Includes setup for baud rate, opening/closing the CAN interface, transmitting frames, and receiving frames.
- **Frame Format**: A typical transmit command might look like `t123811223344` (where `t` is transmit, `123` is the CAN ID, `8` is data length, and `11223344` is the data).

## Usage Example with Python-can
Python-can provides direct support for SLCAN interfaces:
```python
import can

bus = can.interface.Bus(bustype='slcan', channel='/dev/ttyUSB0', receive_own_messages=True)
msg = can.Message(arbitration_id=0x123, data=[0x11, 0x22, 0x33], is_extended_id=False)
bus.send(msg)
```

For more details, refer to the [Python-can SLCAN documentation](https://python-can.readthedocs.io/en/stable/interfaces/slcan.html) or open-source implementations like [SerialCAN on GitHub](https://github.com/mac-can/SerialCAN).

SLCAN is the core protocol used in Doggie for CAN over serial communication.

### hardware/faraday.md
# Faraday Hardware

Faraday Doggie is the official, professionally manufactured hardware version of the Doggie project, released by Faraday Security in 2025. It serves as a modular, flexible, open-source adapter that bridges a computer to a CAN Bus network via USB, with support for BLE and UART extensions.

## Key Features
- **Modular Design**: Compatible with standard CAN tools like SocketCAN and Python-can via SLCAN protocol.
- **Hardware Specs**: Built around supported microcontrollers (e.g., ESP32 variants) with integrated CAN controllers or MCP2515 modules.
- **Use Cases**: Ideal for car hacking, automotive security research, and CAN Bus analysis. For example, it can be used to sniff and inject messages into vehicles like a 2010 Jeep Liberty to unlock doors.
- **Open-Source**: Fully open-source firmware and schematics, available on GitHub.

For DIY builds, see the DIY section. More info at [Faraday Security's Doggie page](https://faradaysec.com/introducing-doggie-your-modular-can-bus-usb-adapter/).

### hardware/diy/intro.md
# DIY Hardware Introduction

Doggie supports DIY builds using common microcontrollers and CAN modules. This modular approach allows customization based on available parts, with options for USB, UART, or BLE connectivity. 

Key components:
- Microcontrollers: RP2040 (Pico), STM32F103C8 (Bluepill), ESP32 variants.
- CAN Controllers: Built-in (e.g., TWAI on ESP32) or external MCP2515 (requires modification for 3.3V compatibility; see MCP2515 Notes).
- Transceivers: MCP2551 or TJA1050 (included in MCP2515 modules).

Refer to the microcontroller-specific guides for connections, flashing, and compilation. All firmware is built with Rust.

### hardware/diy/esp32.md
# ESP32 DIY Configuration

## Description
This implementation provides a **CAN Bus to USB or BLE adapter** using the **ESP32** microcontroller (commonly available in devkits such as the ESP-WROOM-32 we will use as example). The adapter uses the **slcan protocol** (CAN over Serial), making it compatible with popular software tools such as **SocketCAN**, **Python-can**, and other slcan-compatible applications. Since these boards include a UART bridge connected to the UART0 peripheral, SLCAN will be available through UART0 and USB using the default configuration.

## Supported Configurations

The ESP32 implementation supports the following configurations.

As the ESP32 doesn't have 5v tolerant GPIOs, we should modify the MCP2515 or use a logic level shifter in order to make it compatible. Read [MCP2515 module compatibility note](../docs/mcp_mod.md) for more information.

1. **USB, UART0 or BLE, and MCP2515 (SPI to CAN)**
   - The **USB** port, the **UART0** port and the BLE of the ESP32 can be used for communication with the host system.
   - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.
   - This configuration allows the device to interface with a CAN network while communicating with the host via USB or BLE.

    __Connections__ (MCP2515 mod):
    | Function |    MCP2515     |
    | -------- | -------------- |
    |   Vcc 3.3|       VCC      |
    |   Vcc 5v | Tranceiver Vcc |
    |   GND    |       GND      |
    |   MOSI   |       SI       |
    |   MISO   |       SO       |
    |   Clock  |       SCK      |
    |   CS     |       CS       |

    ![alt text](../docs/esp32_mcp_mod.png)

    __Connections__ (MCP2515 with level shifter):
    | Function | Level Shifter | MCP2515 |
    | -------- | ------------- | ------- |
    |   Vcc    |        -      |    5v   |
    |   GND    |        -      |    GND  |
    |   MOSI   | <-----------> |    SI   |
    |   MISO   | <-----------> |    SO   |
    |   Clock  | <-----------> |    SCK  |
    |   CS     | <-----------> |    CS   |

    ![alt text](../docs/esp32_mcp_ls.png)


2. **USB, UART0 or BLE, and TWAI (Internal controller)**
   - The **USB** port, the **UART0** port and the BLE of the ESP32 can be used for communication with the host system.
   - The TWAI controller is used for CAN Bus communication.
   - This configuration allows the device to interface with a CAN network using only a transceiver while communicating with the host via USB or BLE.

    ![alt text](../docs/esp32_twai.png)

    __Connections__:
    | Function |   Tranceiver   |
    | -------- |--------------- |
    |   Vcc    |       VCC      |
    |   GND    |       GND      |
    |   CAN TX |       TX       |
    |   CAN RX |       RX       |


    For each esp32 varian we will use different pins that are defined but they could be easily changed in the code. Some variants are not implemented but are compatible and will be implemented on demand.

   __Connections variants__:
    | Function  |   ESP32  | ESP32c3  |
    | ----------- | -------- | -------- |
    |    Vcc 3.3  |   3v3    |    3.3   |
    |    Vcc 5v   |  VIN/5v  |    5v    |
    |    GND      |   GND    |     G    |
    |    MOSI     |   D13    |     6    |
    |    MISO     |   D12    |     5    |
    |    Clock    |   D14    |     9    |
    |    CS       |   D15    |     7    |
    |    CAN TX   |   D4     |     10    |
    |    CAN RX   |   D3     |     9    |
    | LOGS (UART) |   D10    |     3    |

## Notes on Debugging

The UART-USB or SerialJtagUsb bridge of the esp32 is usually used to flash, write logs and debugging, but as we will be using it as a serial interface for CAN Bus, we need another way to log and debug. For that we set up another UART interface that will print logs. In the "Connections Variants" table we could find the corresponding UART TX pins as **LOGS**.

## How to Flash a Release
1. Install `espflash`
    ```
    cargo install espflash
    cargo install cargo-espflash
    ```
2. Download the release `doggie_esp32`.
3. Run `espflash flash --monitor -L defmt doggie_esp32`

## How to Compile and Flash

### Prerequisites

1. Install **Rust** and **cargo**.
   Follow the installation instructions from the official [Rust website](https://www.rust-lang.org/tools/install).


2. Install `ldproxy`, `espup` and the ESP32 toolchain:
    ```
    cargo install ldproxy
    cargo install espup --version 0.13.0
    espup install --toolchain-version 1.84.0
    . $HOME/export-esp.sh           # Or add to .zshrc/.bashrc
    ```

3. Install `espflash`
    ```
    cargo install cargo-espflash --version 3.2.0
    ```

### Compile and Flash the Firmware

In order to manage the compilation with different hardware variants we use features. The most important feature is the one that select the board (**esp32**, **esp32c3**, etc). And we have the other features:
* `twai`: Enable the internal CAN controller (TWAI)
* `mcp`: Enable the MCP2515 SPI interface as CAN controller
* `ble`: Enable BLE interface

By default `twai` and `ble` are enabled.

1. Connect ESP32 to the PC via USB

3. Build and flash:
    ```bash
    DEFMT_LOG=off cargo {BOARD} --bin {BINARY} --disable-default-features --features {FEATURES}
    ```

    For example:
    ```bash
    # ESP32c3 with TWAI and BLE
    DEFMT_LOG=off cargo esp32c3 --bin doggie

    # ESP32 with MCP2515 and BLE
    DEFMT_LOG=off cargo esp32 --bin doggie --disable-default-features --features mcp,ble
    ```

### hardware/diy/rp.md
# RP2040 (Raspberry Pico) DIY Configuration

## Description  
This implementation provides a **CAN Bus to USB adapter** using the **RP2040** microcontroller (commonly known as **Raspberry Pico** or **Raspberry Pico W**). It supports **two configurations** for interacting with a CAN Bus network, enabling communication via USB or UART. The adapter uses the **slcan protocol** (CAN over Serial), making it compatible with popular software tools such as **SocketCAN**, **Python-can**, and other slcan-compatible applications.

## Supported Configurations

The Raspberry Pico implementation supports the following configurations.

As the RP2040 doesn't have 5v tolerant GPIOs, we shoud modify the MCP2515 or use a logic level shifter in order to make it compatible. Read [MCP2515 module compatibility note](../docs/mcp_mod.md) for more information.

1. **USB and MCP2515 (SPI to CAN)**  
   - The **USB** port of the Pico is used for communication with the host system.  
   - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.  
   - This configuration allows the device to interface with a CAN network while communicating with the host via USB.

    __Connections__ (MCP2515 mod):  
    | Function |   Pico   |    MCP2515     |
    | -------- | -------- | -------------- |
    |   Vcc    |   3.3    |    VCC         |
    |   Vcc 5v |   VBUS   | Tranceiver VCC |
    |   GND    |   GND    |    GND         |
    |   MOSI   |   GP19   |    SI          |
    |   MISO   |   GP16   |    SO          |
    |   Clock  |   GP18   |    SCK         |
    |   CS     |   GP17   |    CS          |

    ![alt text](../docs/pico_mcp_mod.png)

    __Connections__ (MCP2515 with level shifter):  
    | Function |   Pico   | Level Shifter | MCP2515 |
    | -------- | -------- | ------------- | ------- |
    |   Vcc    |   VBUS   |               |    VCC  |
    |   GND    |   GND    |               |    GND  |
    |   MOSI   |   GP19   | <-----------> |    SI   |
    |   MISO   |   GP16   | <-----------> |    SO   |
    |   Clock  |   GP18   | <-----------> |    SCK  |
    |   CS     |   GP17   | <-----------> |    CS   |

    ![alt text](../docs/pico_mcp_ls.png)

2. **UART and MCP2515 (SPI to CAN)**  
   - The **UART** port of the Pico is used to communicate with the host system.  
   - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.  
   - This configuration is useful when the USB port is unavailable or when using a serial connection instead of USB.

    __Connections__ (MCP2515 mod):  
    | Function |   Pico   |    MCP2515     | USB-UART |
    | -------- | -------- | -------------- | -------- |
    |   Vcc    |   3.3    |       VCC      |    -     |
    |   Vcc 5v |   VBUS   | Tranceiver VCC |    5v    |
    |   MOSI   |   GP19   |       SI       |    -     |
    |   MISO   |   GP16   |       SO       |    -     |
    |   Clock  |   GP18   |       SCK      |    -     |
    |   CS     |   GP17   |       CS       |    -     |
    |   TX     |   GP0    |        -       |    RX    |
    |   RX     |   GP1    |        -       |    TX    |   

    ![alt text](../docs/bluepill_uart_mcp.png)

## How to Compile and Flash

### Prerequisites  

probe-rs (para programar con otra pico):
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

1. Install **Rust** and **cargo** with support for ARM architecture.  
   Follow the installation instructions from the official [Rust website](https://www.rust-lang.org/tools/install).  


2. Add the target architecture:
    ```
    rustup target add thumbv6m-none-eabi
    ```

3. To program a Pico, you can put it in bootloader mode and copy the firmware or use another Pico as a probe.

- If you want to program your Pico by copying the firmware, install `elf2uf2-rs`:
    ```
    cargo install elf2uf2-rs
    ```

- If you want to program your Pico using another Pico as a probe, install `probe-rs`:
    ```
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
    ```

    Additionally, you have to modify `doggie_pico/.cargo/config.toml`:
    ```
    [target.'cfg(all(target_arch = "arm", target_os = "none"))']
    # runner = "elf2uf2-rs -d"
    runner = "probe-rs run --chip RP2040"

    [build]
    target = "thumbv6m-none-eabi"        # Cortex-M0 and Cortex-M0+

    [env]
    DEFMT_LOG = "trace"
    ```

    And setup a Pico as a probe by cloning the [RPI debug probe repo](https://github.com/raspberrypi/debugprobe), building it for the pico:
    ```
    git clone https://github.com/raspberrypi/debugprobe.git
    cd debugprobe
    mkdir build
    cd build
    cmake -DDEBUG_ON_PICO=ON ..
    make
    ```
    Copying the resulting `.uf2` image to the Pico, and connecting it to the target Pico.
    
     __Connections__:  
    | Function |   Pico Probe   | Target Probe |
    | -------- | -------------- | ------------ |
    |  Vcc     |      VBUS      |     VBUS     |
    |  GND     |      GND       |     GND      |
    |  SWCLK   |      GP2       |     SWCLK    |
    |  SWDIO   |      GP3       |     SWDIO    |
    |  UART -> |      GP4       |     GP1      |
    |  UART <- |      GP5       |     GP0      |

    ![alt text](../docs/pico_probe.png)

    Note that you will be using the probe as a SWD programer and as a UART bridge, so you must compile doggie using the `uart` feature.

    __Make the modification to the MCP2515 or use a logic level shifter__


### Compile and Flash the Firmware:

To enable BLE and select UART or USB we use features:
* `ble`: Enable Bluetooth Low Energy.
* `uart`: Use UART as serial interface.
* `usb`: Use USB as serial interface.

*Note: one, and only one, serial interface feature (uart or usb) must be selected*

1. Connect the target Pico to the PC in bootloader mode or to the probe as shown before.  

2. Build and flash with selected features
    * USB and MCP2515 with BLE enable:
        ```
        cargo run --release
        ```
    * UART and MCP2515 with ble (Use this feature if you are using the probe):
        ```
        cargo run --release --no-default-features --features uart,ble
        ```

### hardware/diy/bluepill.md
# STM32F1 (Bluepill) DIY Configuration

## Description  
This implementation provides a **CAN Bus to USB adapter** using the **STM32F103C8** microcontroller (commonly known as **Bluepill**). It supports **three configurations** for interacting with a CAN Bus network, enabling communication via USB or UART, while leveraging different CAN transceiver options. The adapter uses the **slcan protocol** (CAN over Serial), making it compatible with popular software tools such as **SocketCAN**, **Python-can**, and other slcan-compatible applications.

## Supported Configurations

The Bluepill implementation supports the following configurations:

1. **USB and MCP2515 (SPI to CAN)**  
   - The **USB** port of the Bluepill is used for communication with the host system.  
   - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.  
   - This configuration allows the device to interface with a CAN network while communicating with the host via USB.

    __Connections__:  
    | Function |  Bluepill  | MCP2515 |
    | -------- | ---------- | ------- |
    |   Vcc    |    5v      |    5v   |
    |   GND    |    GND     |    GND  |
    |   MOSI   |    PB15    |    SI   |
    |   MISO   |    PB14    |    SO   |
    |   Clock  |    PB13    |    SCK  |
    |   CS     |    PB12    |    CS   |

    ![alt text](../docs/bluepill_usb_mcp.png)

2. **UART and MCP2515 (SPI to CAN)**  
   - The **UART** port of the Bluepill is used to communicate with the host system.  
   - The **MCP2515** (SPI to CAN) module is used for CAN Bus communication.  
   - This configuration is useful when the USB port is unavailable or when using a serial connection instead of USB.

    __Connections__:  
    | Function |  Bluepill  | MCP2515 | USB-UART |
    | -------- | ---------- | ------- | -------- |
    |   Vcc    |    5v      |    5v   |    5v    |
    |   GND    |    GND     |    GND  |   GND    |
    |   MOSI   |    PB15    |    SI   |    -     |
    |   MISO   |    PB14    |    SO   |    -     |
    |   Clock  |    PB13    |    SCK  |    -     |
    |   CS     |    PB12    |    CS   |    -     |
    |   TX     |    A2      |    -    |    RX    |
    |   RX     |    A3      |    -    |    TX    |   

    ![alt text](../docs/bluepill_uart_mcp.png)

3. **UART and Internal CAN Controller**  
   - The **UART** port of the Bluepill is used to communicate with the host system.  
   - The internal **CAN controller** of the STM32F103C8 microcontroller is used for CAN Bus communication and one tranceiver (MCP2551 in this case).  
   - **Note:** The Bluepill's **USB port** and **internal CAN controller** cannot be used simultaneously. If the internal CAN controller is selected, the only available communication interface with the host is **UART**.

    __Connections__:  
    | Function | Bluepill | MCP2551 | USB-UART |
    | -------- | -------- | ------- | -------- |
    |   Vcc    |    5v    |    VDD  |    5v    |
    |   GND    |    GND   |    VSS  |   GND    |
    |   CAN TX |    B8    |    TX   |    -     |
    |   CAN RX |    B9    |    RX   |    -     |
    |   RS     |    GND   |    RS   |    -     |
    |   TX     |    A2    |    -    |    RX    |
    |   RX     |    A3    |    -    |    TX    |  

    ![alt text](../docs/bluepill_uart_internal.png)

### Note on MCP2551 compatibility
There is no need to modify the MCP2551 standard module as the bluepill pins selected for the SPI are 5v tolerant.

## How to Flash a Release Using St-Link v2

### Prerequisites

**Installing stlink tools**
```bash
sudo apt update
sudo apt install stlink-tools
```

Alternatively, build the tools from source (if you need the latest version):
```bash
git clone https://github.com/stlink-org/stlink.git
cd stlink
make release
sudo make install
```

## Preparing the Firmware
Ensure your firmware binary is compiled and ready to flash. You could download the release file `doggie_bluepill_{serial}_{can}` with the desired configuration from the [Release](https://github.com/infobyte/doggie/releases) page.

## Flashing the Firmware

1. Connect the ST-LINK programmer to your computer via USB.
  - `ST-LINK SWDIO` → `Bluepill SWDIO`
  - `ST-LINK SWCLK` → `Bluepill SWCLK`
  - `ST-LINK GND` → `Bluepill GND`
  - `ST-LINK 3.3V` → `Bluepill 3.3V` (if not powered externally)

    ![alt text](../docs/bluepill_stlink.webp)

2. Power the Bluepill (either through the ST-LINK or an external power source).
3. Open a terminal and navigate to the folder containing your downloaded firmware binary.
4. Flash the firmware using `st-flash`:
   ```bash
   st-flash write doggie_bluepill_usb_mcp 0x8000000
   ```
   Explanation:
   - `write`: Command to write the firmware.
   - `doggie_bluepill_usb_mcp`: The binary firmware file to be flashed.
   - `0x8000000`: Starting address of the STM32F103C8 flash memory.

## How to Compile and Flash

### Prerequisites  
1. Install **Rust** and **cargo** with support for ARM architecture.  
   Follow the installation instructions from the official [Rust website](https://www.rust-lang.org/tools/install).  


2. Add the target architecture:
    ```
    rustup target add thumbv7m-none-eabi
    ```

3. Install `probe-rs`
    ```
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
    ```

### Compile and Flash the Firmware Using ST-Link V2:

In order to manage the compilation with different hardware variants we use the following features.
* `int`: Enable the internal CAN controller
* `mcp`: Enable the MCP2515 SPI interface as CAN controller
* `usb`: Use the USB as a serial interface
* `uart`: Use the UART as a serial interface

*NOTE: One, and only one, CAN interface feature must be enabled, and the same for the serial interfaces. Also, 'int' and 'usb' can't be enabled at the same time due to a hardware compatibility issue. By default, 'uart' and 'int' are enabled.*

1. Connect Bluepill to the programmer  

2. Build and flash with selected features
    * USB and MCP2515:
        ```
        cargo run --bin doggie --release --no-default-features --features usb,mcp
        ```
    * UART and MCP2515:
        ```
        cargo run --bin doggie --release --no-default-features --features uart,mcp
        ```
    * UART and internal CAN:
        ```
        cargo run --bin doggie --release --no-default-features --features uart,int
        ```

### hardware/mcp.md
### MCP2515 Module Modification Guide

The project supports a variety of hardware configurations, but the most straightforward and widely recommended approach is to use the popular MCP2515 module in conjunction with a compatible microcontroller. While the MCP2515 module is a cost-effective and reliable choice for CAN Bus interfacing, it requires a specific modification to ensure seamless compatibility with most modern microcontrollers. This section explains the compatibility issue in detail, its root cause, and provides two practical solutions to address it.

![alt text](mcp2515.jpg)

#### Background: Understanding the MCP2515 Module
The MCP2515 module is a complete solution for CAN Bus communication, integrating two key components:
1. **MCP2515 CAN Bus Controller**: A standalone controller that communicates with a microcontroller via the Serial Peripheral Interface (SPI) protocol. It is responsible for managing CAN Bus data frames and can operate at either 3.3V or 5V logic levels.
2. **TJA1050 CAN Transceiver**: A high-speed transceiver that interfaces directly with the CAN Bus physical layer, converting the controller’s signals into the differential voltages required by the CAN protocol. The TJA1050, however, operates exclusively at 5V.

These components are typically sold pre-assembled on a single module, with both the MCP2515 controller and the TJA1050 transceiver sharing a single power pin (VCC). This design simplifies the module but introduces a compatibility challenge when interfacing with microcontrollers that operate at 3.3V, as explained below.

![alt text](mcp_diagram.png)

**Datasheet References**:
- MCP2515: [Link to MCP2515 Datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/MCP2515-Stand-Alone-CAN-Controller-with-SPI-20001801J.pdf)
- TJA1050: [Link to TJA1050 Datasheet](https://www.nxp.com/docs/en/data-sheet/TJA1050.pdf)

#### The Compatibility Issue
The MCP2515 module’s single VCC pin forces both the controller and transceiver to operate at the same voltage. While the MCP2515 can function at either 3.3V or 5V, the TJA1050 requires 5V to operate correctly. As a result, the module is typically powered at 5V to satisfy the transceiver’s requirements. However, this causes the MCP2515 to output 5V logic levels on its SPI pins (e.g., MOSI, MISO, SCK, and CS), which are used to communicate with the microcontroller.

Many modern development boards—such as the Raspberry Pi, Arduino models with 3.3V logic, or various ARM-based microcontrollers—operate at 3.3V and do not tolerate 5V logic levels on their GPIO pins. Applying 5V signals to a 3.3V microcontroller can damage the hardware or cause unreliable operation, rendering the unmodified MCP2515 module incompatible with these systems.

#### Solution 1: Using a Logic Level Shifter
The first and simplest solution is to retain the module’s default 5V power configuration and add a logic level shifter between the MCP2515 module and the microcontroller. This approach works as follows:
- **Power the MCP2515 Module at 5V**: Connect the module’s VCC pin to a 5V supply, ensuring both the MCP2515 controller and TJA1050 transceiver function correctly.
- **Add a Logic Level Shifter**: Place a bidirectional logic level shifter (e.g., based on a 74LVC series IC or a MOSFET-based shifter) between the SPI pins of the MCP2515 module (operating at 5V) and the microcontroller’s GPIO pins (operating at 3.3V). This converts the 5V logic signals from the MCP2515 to 3.3V signals compatible with the microcontroller, and vice versa.

![alt text](mcp_ls.png)

**Advantages**:
- No physical modification to the module is required, preserving its original state.
- Straightforward to implement with widely available, inexpensive components.

**Disadvantages**:
- Adds extra hardware complexity and cost to the project.
- Requires careful wiring to ensure proper signal integrity.

**Implementation Steps**:
1. Power the MCP2515 module with a 5V supply.
2. Connect a logic level shifter to the SPI lines (MOSI, MISO, SCK, CS) between the module and the microcontroller.
3. Verify the shifter’s low-voltage side is connected to 3.3V and the high-voltage side to 5V, per the shifter’s specifications.
4. Test the setup to confirm reliable SPI communication.

#### Solution 2: Modifying the MCP2515 Module
A second, more hardware-oriented solution involves physically modifying the MCP2515 module to separate the power supplies for the MCP2515 controller and the TJA1050 transceiver. This allows the controller to operate at 3.3V (matching the microcontroller’s logic levels) while the transceiver remains at 5V. Here’s how it works:
- **Desolder the TJA1050 VCC Pin**: On the MCP2515 module, locate the TJA1050 transceiver’s VCC pin (typically connected to the module’s shared VCC line). Carefully desolder this pin to disconnect it from the module’s power rail.
- **Add a Separate 5V Supply**: Solder a small wire from the TJA1050’s VCC pin to an external 5V power source, ensuring the transceiver receives the required voltage.
- **Power the MCP2515 at 3.3V**: Connect the module’s VCC pin (now powering only the MCP2515 controller) to a 3.3V supply. This configures the MCP2515’s SPI pins to output 3.3V logic levels, making it directly compatible with 3.3V microcontrollers.

![alt text](mcp_mod.jpg)

**Advantages**:
- Eliminates the need for an external logic level shifter, reducing component count.
- Provides a clean, direct 3.3V interface with the microcontroller.

**Disadvantages**:
- Requires soldering skills and careful modification of the module, which may risk damage if done incorrectly.
- Slightly more complex to document and replicate for end users.

**Implementation Steps**:
1. Identify the TJA1050 transceiver on the MCP2515 module and locate its VCC pin (refer to the module schematic or TJA1050 datasheet).
2. Using a soldering iron and desoldering tools (e.g., braid or pump), carefully lift the TJA1050 VCC pin from the shared power trace.
3. Solder a thin wire (e.g., 30 AWG) from the lifted VCC pin to a 5V power source.
4. Connect the module’s VCC pin to a 3.3V supply.
5. Verify the modification with a multimeter to ensure the MCP2515 receives 3.3V and the TJA1050 receives 5V.
6. Test the module with a 3.3V microcontroller to confirm SPI communication and CAN Bus functionality.

#### Recommendation

We recommend **Solution 2 (module modification)** as the preferred approach, as it aligns with the goal of creating a seamless, plug-and-play CAN Bus adapter compatible with a wide range of microcontrollers. While it requires more effort upfront, it simplifies the end-user experience by eliminating the need for additional components. However, for users uncomfortable with soldering, **Solution 1 (logic level shifter)** remains a viable alternative and should be documented as an option.

#### Conclusion
The MCP2515 module is an excellent choice for CAN Bus interfacing, but its default configuration poses a voltage compatibility challenge with 3.3V microcontrollers. By implementing one of the above solutions—either a logic level shifter or a physical modification—users can ensure reliable operation with the "Doggie" USB adapter. Both methods have been tested and validated for this project, and detailed schematics, photos, or diagrams of the modification process can be provided upon request.

### workshop/intro.md
# evilDoggie Workshop Introduction

## Workshop Overview
This workshop is designed to introduce participants to Doggie and its offensive variant, evilDoggie. Rather than teaching how a real car works, the workshop focuses on using Doggie and evilDoggie to analyze and manipulate a simulated CAN Bus environment specially built for training. Through several progressive challenges, participants will learn how Doggie integrates with standard tools and how, using evilDoggie, an attacker could take advantage of this protocol. For more information about CAN protocol and CAN hacking, check the appendix at the end of this guide.

## Content Outline
- Introduction
- Doggie and evilDoggie
  - What is Doggie?
  - What is evilDoggie?
- Get Started
  - Good Mode
    - Install tools
    - Create the CAN interface
    - Bring up the interface
    - Start sniffing with candump
  - Evil Mode
    - The logic behind evilDoggie attacks
    - How this workshop works
    - How it is physically connected
- Challenges
  - 1. VIN Spoofing (details in full guide)
  - Speed Spoofing: Use spoofing_attack to set speed to 0.
  - Door Unlock: Handle bus contention with higher priority messages.
  - Airbag Disable: Use double receive attack by injecting errors in EOF field.
  - Bus Off Attack: Force target ECU offline by injecting errors.
  - Engine Start without Key: Use force mode to override key status at physical level.

## Detailed Challenges

### Challenge: Speed Spoofing
The car’s speed is 0 km/h. Instead of just sniffing or sending normal messages, here you’ll need to use evilDoggie, the offensive firmware variant of Doggie. You’ll apply a classic attack known as spoofing, where you send fake messages with speed = 0.

evilDoggie provides a special spoofing_attack command:
> spoofing_attack 0x100 0x2,0x0,0x0 0x2

- 0x100: CAN ID
- 0x2,0x0,0x0: data (speed = 0)
- 0x2: mask (spoof only other messages on the same ID)

Run the attack multiple times with > attack 20.

Observe: The accelerator goes to 0% as the CC ECU now knows the actual speed.

### Challenge: Door Unlock
Here’s how to build and run this attack step by step with evilDoggie.

First, open evilDoggie in your terminal.

The first part of the attack: Notice how a GoodDoggie tries to send a message with higher priority, just as the evilDoggie is trying to send a door unlock message. This results in a bus error.

A successful one looks like this: In this case, there is no contention for the bus. The evilDoggie sends the door unlock message successfully, and is ACK’ed by the GoodDoggies.

This can be avoided using the bus takeover feature of evilDoggie. By taking full control of the bus and silencing other ECUs, the attacker can ensure that their messages go through.

### Challenge: Airbag Disable
Here’s how to do this step by step with evilDoggie.

To set up a double receive attack, use: Toggle the airbag button on the simulator UI.

If the attack succeeds:
- evilDoggie’s console will log that the double receive attack was launched.
- The Instrument Cluster will show the airbag disabled.

Observe the result in the logic analyzer: evilDoggie injects an error just after the 6th bit of the EOF field in the CAN frame corresponding to the Airbag.

### Challenge: Bus Off Attack
Here’s how to do this attack using evilDoggie.

> bus_off_attack 0x105 50

evilDoggie will wait for messages with ID 0x105 (ABS heartbeat) and inject errors, effectively silencing the target ECU for a while.

Observe: The evilDoggie injecting errors after each ABS heartbeat message causing the ABS ECU to go silent for a period.

### Challenge: Engine Start without Key
The engine can’t start unless the Immo ECU reports the key is present. You’ll use a unique feature of evilDoggie called force to override the messages that report the key status at the physical level.

evilDoggie’s force mode (also called dominant-override) can:
- Take control of the bus.
- Force recessive bits to dominant at the physical level.

This keeps the Immo ECU from sending “key not present” messages and allows evilDoggie to send forged “key is present” messages, followed by a start command.

Solution: We’ll build a custom attack in evilDoggie using the force feature.

Enter the custom attack menu:
> custom_attack

First, wait until the attack succeeded.

Congratulations—you’ve completed the final and most advanced challenge, using evilDoggie’s physical-layer force capability to break the car’s security and start the engine without a key!

Observe in the logic analyzer: evilDoggie is forcing the KeyMessage indicating that the key is present. Depending on the timing you might see the RX line does not match the data its trying to send, it enters into an error state. Finally, evilDoggie sends the message to start the car.

## Appendix: CAN Protocol
The CAN protocol is always evolving. You can build your own DIY Doggie by combining parts you already have! For more information on this, check the appendix on CAN protocol at the end of this guide. And that’s exactly why we’ll use tools like Doggie and evilDoggie to see how these protocol details can be turned into practical attacks.
