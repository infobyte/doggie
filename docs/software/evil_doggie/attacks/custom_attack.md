# Custom Attack

## Description

The `custom_attack` submenu allows users to construct low-level attacks with bit-level precision by chaining primitives. Predefined attacks are built from these primitives, but custom attacks offer granular control for complex scenarios, such as forcing an engine start. It supports primitives like `send_msg`, `wait_bus_free`, and `send_error` to tailor attacks to specific vulnerabilities.

## Console Usage

1. Access the main menu via the serial console.
2. Enter the custom attack submenu with `custom_attack` (prompt changes to `custom_attack>`).
3. Build the attack using the following primitives:
    - **match_id <id> [ --extended ]**: Waits for a message with the specified CAN ID (e.g., `match_id 0x300 --extended` for a 29-bit ID).
    - **match_data <dlc> [ <data> ]**: Filters by data length and content (e.g., `match_data 2 0x00,0x00` matches 2 bytes starting with `0x00,0x00`).
    - **skip_data**: Ignores data fields, matching only ID and DLC (e.g., `skip_data`).
    - **wait <bits>**: Inserts a delay in CAN bit times (e.g., `wait 10` for 10 bits).
    - **send_error <count>**: Sends consecutive error frames (e.g., `send_error 3` for 3 errors).
    - **send_raw <bits> [ --force ]**: Transmits a raw bit sequence (e.g., `send_raw 0110001 --force` with forced override).
    - **wait_bus_free**: Pauses until the bus is idle (e.g., `wait_bus_free`).
    - **send_msg <id> [ --extended ] [ --rtr ] [ --force ] [ <data> ]**: Sends a CAN frame (e.g., `send_msg 0x400 0x01 --force` for a forced start command).
    - **set_bitstuffing <state>**: Enables/disables bitstuffing (e.g., `set_bitstuffing off`).
4. Example: To force an engine start:
    - `match_id 0x300`
    - `match_data 2 0x00,0x00`
    - `wait 10`
    - `send_msg 0x400 0x01 --force`
    - `save "engine_start"`
5. Manage the sequence with `list`, `delete <index>`, or `move <from> <to>`, then `exit` to return to the main menu.
6. Add to the plan with `add_custom engine_start` and execute with `attack` (e.g., `attack 1`).
7. Use `help` or `help <command>` (e.g., `help send_msg`) for primitive details or assistance.

## How it works

`TODO`
