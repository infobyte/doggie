// ROOT MENU
pub const ENTER_ROOT_TXT: &'static str = "
Hi! You're now in the main menu of EvilDoggie.
Here you can build your attack plan by chaining one or more predefined attacks using the *_attack commands.

You can also review or adjust your attack plan at any time with:
  * list to see the current plan
  * move to change the order of steps
  * remove to delete an attack from the plan

When your plan is ready, launch it with:
  * attack to start the execution

And remember: the help command will always show you available commands and options. Happy hacking!
";

// SET BAUDRATE COMMAND
pub const SET_BAUDRATE_TXT: &'static str = "Configure the CAN bus bitrate for the device. Use this to match the baudrate of the target CAN network before running attacks";

pub const SET_BAUDRATE_PARAM_TXT: &'static str =
    "Desired CAN bus speed in kbps. Valid options are: 5, 10, 20, 50, 100, 125, 250, 500, or 1000.";

// CUSTOM ATTACK COMMAND
pub const CUSTOM_ATTACK_TXT: &'static str = "Enter the custom attack menu to craft your own low‑level attacks with bit‑level precision.

Internally, every predefined attack in EvilDoggie is built by chaining together attack primitives, basic building blocks like sending forced messages, waiting for bus conditions, or injecting errors.

In the custom attack submenu, you can manually combine these primitives in any order to create a custom attack tailored to your goal. When finished:
  * Use save to add your custom attack to the current attack plan.
  * Use exit to exit to return to the main menu.";

pub const ENTER_CUSTOM_ATTACK_TXT: &'static str = "Here you can craft your own low‑level attack by chaining together attack primitives like sending messages, waiting for bus conditions, or forcing bits on the bus.

Use commands like send_msg, wait_bus_free, and others to build your custom attack step by step.
When you’re done, type save to add it to your attack plan or exit to go to the main menu.

Type help at any time to see the full list of available primitives and commands.";

// SPOOFING ATTACK
pub const SPOOFING_ATTACK_TXT: &'static str = "Push to the plan a spoofing attack to injecting a forged message on the bus immediately after seeing a specific real message.

This works by monitoring the bus in real‑time and, when a matching message is detected (using <id> and optional <match_data>), EvilDoggie quickly sends your crafted <spoofed_data> message.";

pub const SPOOFING_ATTACK_DATA_TXT: &'static str =
    "Data bytes to spoof as comma-separated hex values (e.g., 0x10,0x20,0x30)";

// COMMON PARAMS
pub const PARAM_MATCH_ID_TXT: &'static str = "CAN ID to match, in hex (e.g., 0x123, 0x12345678).";

pub const PARAM_MATCH_DATA_TXT: &'static str = "Match only after seeing a real message on the bus with the same ID and whose first data bytes match these comma‑separated hex values. Useful to target specific messages when multiple messages share the same ID.";

pub const PARAM_EXTENDED_TXT: &'static str =
    "Use if the target ID is an extended 29‑bit ID. Defaults to standard 11‑bit IDs.";

pub const PARAM_ERRORS_TXT: &'static str = "Number of consecutive error frames to inject.";

pub const PARAM_SEND_ID_TXT: &'static str = "CAN ID to send, in hex (e.g., 0x123, 0x12345678).";

pub const PARAM_FORCE_TXT: &'static str = "Override bus state while sending.";

// BUS OFF ATTACK
pub const BUS_OFF_ATTACK_TXT: &'static str = "Push a bus‑off attack that monitors the CAN bus and, when a matching message is detected, injects a burst of error frames timed to disrupt communication.

By sending repeated errors during transmission, this attack can force a target node’s CAN controller into a bus off state or significantly delay its messages, making it temporarily stop communicating or degrade its performance.";

// DOUBLE RECEIVE ATTACK
pub const DOUBLE_RECEIVE_ATTACK_TXT: &'static str = "Configure a double receive attack that injects error frames with precise timing to cause the sender to retransmit a frame while the receiver still accepts the first one.

This results in the receiver processing the same message twice, which can confuse state machines or logic relying on single delivery.";

pub const LIST_PLAN_TXT: &'static str =
    "Display the current attack plan, showing all configured attacks and their order.";

pub const MOVE_ATTACK_TXT: &'static str = "Reorder attacks in the current attack plan.
Move the attack currently at position <from_index> to the new position <to_index>.
Index numbers start from 0 (the first attack).";

pub const DELETE_ATTACK_TXT: &'static str =
    "Delete the attack at the specified index from the current attack plan.
Use this to clean up or adjust your plan before execution.";

pub const DELETE_ATTACK_INDEX_TXT: &'static str = "Index of the attack to delete";

pub const ATTACK_TXT: &'static str = "Start executing the current attack plan you’ve built.
The attack will run until it achieves the specified number of successful executions or the maximum retries limit is reached.";

pub const ATTACK_SUCCESSES_TXT: &'static str =
    "Number of successful attack executions to achieve before stopping. Defaults to 1.";

pub const ATTACK_RETIRES_TXT: &'static str = "Maximum number of retries allowed if the attack fails before giving up. Defaults to infinite retries.";

// match_id
pub const AC_MATCH_ID_TXT: &'static str = "Wait until a message with the specified CAN ID appears on the bus before continuing to the next primitive.
Match as an extended (29‑bit) ID instead of standard (11‑bit).";

// match_data
pub const AC_MATCH_DATA_TXT: &'static str = "After matching a message ID, further filter messages by data length and content before continuing. If dlc > len(data) will match data partially (e.g., dlc = 3 and data 0x10,0x20 will match frames whith data starting 0x10,0x20 and any value for the 3rd byte.";
pub const AC_MATCH_DATA_DLC_TXT: &'static str =
    "Data Length Code: number of bytes the message must have.";
pub const AC_MATCH_DATA_DATA_TXT: &'static str =
    "Comma‑separated hex values to match against the first bytes of the data payload.";

// skip_data
pub const AC_SKIP_DATA_TXT: &'static str =
    "Ignore the data field when matching messages; only the CAN ID and DLC will be checked.";

// wait
pub const AC_WAIT_TXT: &'static str =
    "Insert a precise delay measured in CAN bit times to control the timing of your attack.";
pub const AC_WAIT_BITS_TXT: &'static str =
    "Number of bit times to wait before executing the next primitive.";

// send_error
pub const AC_SEND_ERROR_TXT: &'static str =
    "Send error frames on the bus to disrupt or delay message transmission.";
pub const AC_SEND_ERROR_COUNT_TXT: &'static str = "Number of consecutive error frames to send.";

// send_raw
pub const AC_SEND_RAW_TXT: &'static str =
    "Transmit a raw custom bit pattern onto the bus, with optional force to override bus state.";
pub const AC_SEND_RAW_BITS_TXT: &'static str = "Raw bit sequence to send, e.g., 0110001.";

// wait_bus_free
pub const AC_WAIT_BUS_FREE_TXT: &'static str =
    "Pause execution until the CAN bus becomes idle (no dominant bits on the bus).";

// send_msg
pub const AC_SEND_MSG_TXT: &'static str =
    "Send a complete CAN frame onto the bus, optionally forcing it or using extended IDs.";
pub const AC_SEND_MSG_DATA_TXT: &'static str =
    "Comma‑separated hex bytes to send as the data payload.";
pub const AC_SEND_MSG_RTR_TXT: &'static str = "Set rtr bit on the message frame.";

// set_bitstuffing
pub const AC_SET_BITSTUFFING_TXT: &'static str =
    "Control whether bitstuffing is applied when sending raw bits.";
pub const AC_SET_BITSTUFFING_STATE_TXT: &'static str =
    "Enable or disable bitstuffing; use 'on' or 'off'.";

// delete
pub const AC_DELETE_TXT: &'static str = "Remove a primitive from your custom attack.";
pub const AC_DELETE_IDX_TXT: &'static str =
    "Index of the primitive in your current custom attack plan to remove.";

// move
pub const AC_MOVE_TXT: &'static str = "Change the order of primitives in your custom attack plan.";
pub const AC_MOVE_FROM_TXT: &'static str = "Current index of the primitive.";
pub const AC_MOVE_TO_TXT: &'static str = "New index to move it to.";

// list
pub const AC_LIST_TXT: &'static str =
    "Display all primitives currently added to your custom attack plan in execution order.";
pub const AC_SAVE_AND_EXIT_TXT: &'static str = "Add this custom attack to the plan.";
