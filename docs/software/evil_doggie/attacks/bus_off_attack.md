# Bus Off Attack
- **Command**: `bus_off_attack <ID> <error_count>`
  - `<ID>`: The CAN ID to target (e.g., `0x105` for an ABS heartbeat).
  - `<error_count>`: Number of errors to inject (e.g., `50` to force bus off).
- **Description**: This attack floods the target ECU with errors by injecting bit errors into frames with the specified ID. After accumulating enough errors (typically 255 per CAN spec), the ECU enters a "bus off" state, disconnecting from the network.
- **Example**: `bus_off_attack 0x105 50` targets an ABS ECU, causing it to go offline by injecting errors after each heartbeat.
- **Use Case**: Used to silence specific ECUs, such as disabling ABS or other safety systems.
