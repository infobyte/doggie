# Custom Attacks in evilDoggie

The `custom_attack` submenu allows you to craft sophisticated attacks by combining low-level primitives tailored to specific CAN Bus manipulation needs. These attacks leverage the physical layer and protocol weaknesses, offering flexibility beyond predefined options. Here’s how to create and manage custom attacks:

- **Entering Custom Attack Mode**: Type `custom_attack` from the main menu to access this submenu. The prompt changes to `>>` to indicate you’re in custom attack mode.
- **Available Primitives**:
  - `wait_frame <ID>`: Waits for a CAN frame with the specified ID (e.g., `wait_frame 0x100`) before proceeding with the next step.
  - `inject_error <bit_position>`: Injects an error at the specified bit position in the next detected frame (e.g., `inject_error 6` targets the EOF field).
  - `force_dominant <bit_range>`: Forces bits to dominant (logic 0) over a specified range in a frame (e.g., `force_dominant 0-2` overrides the first three bits).
  - `send_frame <ID> <data>`: Sends a custom CAN frame with the given ID and data (e.g., `send_frame 0x123 0x11223344`).
  - `delay <ms>`: Introduces a delay in milliseconds (e.g., `delay 100` pauses for 100ms).
- **Building a Custom Attack**:
  - Example: To disable an airbag by triggering a double receive attack:
    1. `wait_frame 0x200` (wait for the airbag status frame).
    2. `inject_error 6` (inject error in the EOF field to cause a double receive).
    3. `save "airbag_disable"` (save the sequence as "airbag_disable").
  - Example: To start an engine without a key:
    1. `wait_frame 0x300` (wait for the key status frame).
    2. `force_dominant 0-7` (override the entire byte to dominant, simulating key presence).
    3. `send_frame 0x400 0x01` (send engine start command).
    4. `save "engine_start"`
- **Managing Custom Attacks**:
  - `list_custom`: Displays all saved custom attacks.
  - `delete_custom <name>`: Removes a custom attack (e.g., `delete_custom airbag_disable`).
  - `edit_custom <name>`: Re-enters edit mode for an existing custom attack to modify its sequence.
  - `exit`: Returns to the main menu, saving the current custom attack if `save` was used.
- **Executing Custom Attacks**: After saving, return to the main menu and add the custom attack to the plan with `add_custom <name>` (e.g., `add_custom airbag_disable`). Launch it with `attack [count]`.

Custom attacks are powerful for targeting specific vulnerabilities, such as overriding key checks or silencing ECUs, and can be iterated upon based on real-time CAN Bus analysis.
