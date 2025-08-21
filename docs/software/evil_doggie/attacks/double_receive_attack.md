# Double Receive Attack

## Description

The `double_receive_attack` exploits CAN error handling by injecting timed error frames, causing the sender to retransmit while the receiver processes the initial frame. This results in duplicate message processing, potentially disrupting state machines or disabling features like airbags by confusing the ECU.

## Console Usage

  1. Connect to the main menu via the serial console.
  2. Add the attack with `double_receive_attack <id> <errors> [ <match_data> ] [ --extended ]`.
     - Example: `double_receive_attack 0x200 3 0x10`
       - Targets ID `0x200`, injecting 3 errors after data starting with `0x10` to trigger a double receive.
  3. Check the plan with `list`, then execute with `attack` (e.g., `attack 1` for one attempt).
  4. See `help double_receive_attack` for parameter specifics.

## How it works

`TODO`
