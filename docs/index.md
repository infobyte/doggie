# Doggie and evilDoggie Project Overview

<figure align="center">
    <img src="res/logo_inv.png" alt="Doggie logo">
</figure>

<figure align="center">
    <img src="res/banner.svg" alt="Doggie banner">
</figure>


**Doggie** is an open-source, modular project designed to build a DIY CAN Bus to serial adapter (USB, BLE, UART). It connects your computer to a CAN Bus network using the slcan protocol (CAN over Serial) for compatibility with tools like SocketCAN and Python-can. The project emphasizes modularity, supporting various microcontrollers (e.g., RP2040, STM32F103C8, ESP32) and CAN controllers (built-in or MCP2515).

**evilDoggie** is the offensive firmware variant of Doggie, tailored for automotive security research and low-level CAN Bus manipulation. It enables advanced attacks like spoofing, bus off, double receive, and physical-layer overrides, making it ideal for red teaming and vulnerability testing in simulated or real CAN environments.

Developed by Faraday Security, the project was presented at Black Hat Arsenal on August 6-7, 2025, in Las Vegas. Both variants are open-source under the MIT License and available on GitHub at [https://github.com/infobyte/doggie](https://github.com/infobyte/doggie). Use responsibly for research and training only.

For detailed introductions, see the sub-sections for [Doggie](/software/doggie/intro) and [evilDoggie](/software/evil_doggie/intro).

