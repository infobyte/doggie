
# **Doggie Bluepill**


## **Description**  
This implementation provides a **CAN Bus to USB adapter** using the **STM32F103C8** microcontroller (commonly known as **Bluepill**). It supports **three configurations** for interacting with a CAN Bus network, enabling communication via USB or UART, while leveraging different CAN transceiver options. The adapter uses the **slcan protocol** (CAN over Serial), making it compatible with popular software tools such as **SocketCAN**, **Python-can**, and other slcan-compatible applications.

---

## **Supported Configurations**

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

---

### Note on MCP2551 compatibility ###
There is no need to modify the MCP2551 standard module as the bluepill pins selected for the SPI are 5v tolerant.


## **How to flash a release using St-Link v2** ##

### Prerequisites ###

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



## **How to Compile and Flash**

### **Prerequisites**  
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

### **Compile and Flash the Firmware Using ST-Link V2:**

1. Connect Bluepill to the programmer  

2. Build and flash with selected features
    * USB and MCP2515:
        ```
        cargo run --release --bin doggie_bluepill_usb_mcp
        ```
    * UART and MCP2515:
        ```
        cargo run --release --bin doggie_bluepill_uart_mcp
        ```
    * UART and internal CAN:
        ```
        cargo run --release --bin doggie_bluepill_uart_int
        ```
