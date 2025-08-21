# Double Receive Attack
- **Command**: `double_receive_attack <ID> <bit_position>`
  - `<ID>`: The CAN ID to target (e.g., `0x200` for airbag status).
  - `<bit_position>`: The bit position to inject an error (e.g., `6` in the EOF field).
- **Description**: This attack exploits the CAN protocol's error handling by injecting a bit error at a specific position (typically the EOF field) in a frame. This causes the receiving ECU to detect a double receive (reception of the same frame twice due to error recovery), potentially leading to misinterpretation or disabling of the targeted function.
- **Example**: `double_receive_attack 0x200 6` injects an error in the airbag status frame's EOF, triggering a double receive to disable the airbag.
- **Use Case**: Effective for disabling safety features like airbags by confusing ECUs with repeated frame reception.
