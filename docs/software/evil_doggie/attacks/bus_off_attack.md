# Bus Off Attack

## Description

The `bus_off_attack` disrupts a target ECU by injecting a series of error frames upon detecting a matching message. This floods the bus, driving the target node's CAN controller into a "bus off" state (after 255 errors per CAN specification), effectively disabling or delaying its communication. Useful for silencing systems like ABS.

## Console Usage

1. Enter the main menu via the serial console.
2. Add the attack with `bus_off_attack <id> <errors> [ <match_data> ] [ --extended ]`.
       - Example: `bus_off_attack 0x105 50 0xFF`
           - Targets ID `0x105`, injecting 50 errors when data starts with `0xFF`.
3. Review the plan with `list`, then launch with `attack` (e.g., `attack 5` for 5 bursts).
4. Consult `help bus_off_attack` for additional options or clarification.

## How it works

`TODO`
