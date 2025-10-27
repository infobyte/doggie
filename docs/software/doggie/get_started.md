# Doggie Get Started

## Prerequisites

If you want to build the project, you will need Rust and Cargo.
Follow the installation instructions from the official [Rust website](https://doc.rust-lang.org/book/ch01-01-installation.html).

The instructions of how to build and flash Doggie are in the corresponding file of each possible configuration, as
it depends on the microcontroller. For more information check `/hardware/diy/{microcontroller}`.

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

The `-sX` argument determines the speed:  

* s0: 10 kbit/s
* s1: 20 kbit/s
* s2: 50 kbit/s
* s3: 100 kbit/s
* s4: 125 kbit/s
* s5: 250 kbit/s
* s6: 500 kbit/s
* s7: 800 kbit/s
* s8: 1 Mbit/s

The `-S{baudrate}` determines the serial interface baudrate (Not necessary on most USB implementations)

```bash
# Start the slcan daemon:
sudo slcand -o -s5 -S115200 /dev/ttyUSB0 doggie0

# Set the interface UP
sudo ifconfig doggie0 up txqueuelen 500
```

### 3. Send/Receive CAN Messages  
- **Send a CAN message:**  
  ```
  cansend doggie0 123#11223344
  ```

- **Receive CAN messages:**  
  ```
  candump doggie0
  ```

For more advanced commands, refer to the [SocketCAN documentation](https://www.kernel.org/doc/Documentation/networking/can.txt).

## BLE  

As some boards supports BLE to send and receive serial information we need a way to bridge the BLE data to a serial interface implementing the NUS service.
Visit the [BLE notes](/software/ble) for mor information.

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
