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

For more details, refer to the [Python-can SLCAN documentation](https://python-can.readthedocs.io/en/stable/interfaces/slcan.html).

SLCAN is the core protocol used in Doggie for CAN over serial communication.
