# Spoofing Attack

## Description

The `spoofing_attack` enables injection of a forged CAN message immediately following the detection of a matching real message. It monitors the bus in real-time for a specified ID (with an optional data match) and transmits the spoofed data to override or augment the original frame. This is effective for altering sensor data (e.g., speed) or control signals to mislead ECUs.

## Console Usage

1. Access the main menu via the serial console.
2. Add the attack to the plan with `spoofing_attack <id> <spoofed_data> [ <match_data> ] [ --extended ]`.
    - Example: `spoofing_attack 0x100 0x00,0x00,0x00 0x01,0x02 --extended`
        - Targets ID `0x100` (29-bit extended) with spoofed data `0x00,0x00,0x00`, triggered by a match with data starting `0x01,0x02`.
3. Verify the plan with `list`, then execute with `attack` (e.g., `attack 10` for 10 iterations).
4. Refer to `help spoofing_attack` for parameter details or troubleshooting.

## How it works

`TODO`
