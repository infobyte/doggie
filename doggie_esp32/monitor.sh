#!/bin/bash

# Check if a serial port argument is provided
if [ -z "$1" ]; then
    echo "Error: No serial port provided."
    echo "Usage: $0 <serial_port>"
    echo "Example: $0 /dev/ttyUSB10"
    exit 1
fi

# Assign the provided serial port
SERIAL_PORT="$1"

# Check if the serial port exists
if [ ! -e "$SERIAL_PORT" ]; then
    echo "Error: Serial port $SERIAL_PORT does not exist!"
    exit 1
fi

# Path to the ELF file
ELF_PATH="./target/riscv32imc-unknown-none-elf/release/doggie_esp32"

# Check if the ELF file exists
if [ ! -f "$ELF_PATH" ]; then
    echo "Error: ELF file $ELF_PATH not found!"
    exit 1
fi

# Run defmt-print with the specified serial port and ELF file
echo "Monitoring $SERIAL_PORT with defmt-print..."
SERIAL_PORT="$SERIAL_PORT" defmt-print -e "$ELF_PATH" serial
