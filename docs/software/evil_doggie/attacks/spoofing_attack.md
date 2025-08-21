# Spoofing Attack
- **Command**: `spoofing_attack <ID> <data> <mask>`
  - `<ID>`: The CAN ID to spoof (e.g., `0x100`).
  - `<data>`: The data payload to send (e.g., `0x2,0x0,0x0` for speed = 0).
  - `<mask>`: A bitmask to filter which bits to spoof (e.g., `0x2` to target specific data fields).
- **Description**: This attack sends fake CAN messages to override legitimate ones with the specified ID. It monitors the bus for the target ID, then injects the provided data when a matching frame is detected. The mask ensures only the intended data fields are altered, preserving other frame content.
- **Example**: `spoofing_attack 0x100 0x2,0x0,0x0 0x2` spoofs a speed message to set speed to 0, useful for simulating a stopped vehicle.
- **Use Case**: Ideal for manipulating sensor data (e.g., speed, RPM) to deceive ECUs.
