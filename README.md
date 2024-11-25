# **Doggie - CAN Bus to USB Bridge**

![alt text](./docs/logo_m.png)

## **Description**  
**Doggie** is an open-source, modular project designed to build a DIY CAN Bus-to-USB adapter. The device connects your computer to a CAN Bus network and uses the **slcan protocol** (CAN over Serial) to ensure compatibility with popular tools like **SocketCAN**, **Python-can**, and other slcan-compatible software.  

The project emphasizes **modularity**, allowing users to select from various hardware configurations with different microcontrollers and CAN transceivers, making it accessible and cost-effective. Whether you're using a microcontroller's built-in CAN controller or an **MCP2515** (SPI to CAN) module, **Doggie** adapts to your needs.

---

## **Supported Configurations**  

### Microcontrollers:
- **Raspberry Pi Pico (1 and 2)**:  `doggie_pico`
- **STM32F103C8 (Bluepill)**: `doggie_bluepill`
- **ESP32**: `doggie_esp32`

### CAN Controllers:  
- Built-in CAN controllers (if supported by the microcontroller)  
- **MCP2515** (SPI to CAN)  

### USB/Serial Connectivity:
- **Microcontroller USB** (native USB support)
- **UART with USB Bridge**  

Each hardware configuration is detailed in its respective subdirectory under the `doggie_{bsp}` folder.

---

## **Get Started** ###


### Prerequisites ###

If you want to build the project, you will need Rust and Cargo.
Follow the installation instructions from the official [Rust website](https://doc.rust-lang.org/book/ch01-01-installation.html).

The instructions of how to build and flash Doggie are in the README.md of each possible configuration, as
it depends on the microcontroller. For more information check `doggie_{bsp}/README.md`.

---

## **Using SocketCAN on Linux**  

SocketCAN is a powerful framework for interfacing with CAN networks. Once the device is connected, follow these steps:

### **1. Install CAN Utilities**  
```bash
# On Ubuntu
sudo apt-get install can-utils
```

### **2. Attach the Device**  
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
* The `-S{baudrate}` determines the serial interface baudrate (Not nessesary on most USB implementations)

```bash
# Strart the slcan daemon:
sudo slcand -s5 -S115200 /dev/ttyUSB0 can0

# Set the interface UP
sudo ifconfig can0 up
```

### **3. Send/Receive CAN Messages**  
- **Send a CAN message:**  
  ```bash
  cansend can0 123#11223344
  ```

- **Receive CAN messages:**  
  ```bash
  candump can0
  ```

For more advanced commands, refer to the [SocketCAN documentation](https://www.kernel.org/doc/Documentation/networking/can.txt).

---

## **Disclaimer**  
This project is a **work in progress**, and contributions are highly encouraged! While it is functional, some features may still be under development.  

If you encounter issues or have suggestions for improvements, please feel free to open an issue or submit a pull request.  

---

<!-- ### **License**   -->
<!-- This project is licensed under the MIT License. See the `LICENSE` file for details. -->
