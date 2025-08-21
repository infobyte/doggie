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
