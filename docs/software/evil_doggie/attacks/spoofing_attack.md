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

## Help

```
> help spoofing_attack

SUMMARY:
  spoofing_attack <id> <spoofed_data> [ <match_data> ] [ --extended ]

PARAMETERS:
  <id>
    CAN ID to match, in hex (e.g., 0x123, 0x12345678).

  <spoofed_data>
    Data bytes to spoof as comma-separated hex values (e.g., 0x10,0x20,0x30)

  <match_data>
    Match only after seeing a real message on the bus with the same ID and whose first data bytes match these comma‑separated hex values. Useful to target specific messages when multiple messages share the same ID.

  --extended
    Use if the target ID is an extended 29‑bit ID. Defaults to standard 11‑bit IDs. 

DESCRIPTION:
Push to the plan a spoofing attack to injecting a forged message on the bus immediately after seeing a specific real message.

This works by monitoring the bus in real‑time and, when a matching message is detected (using <id> and optional <match_data>), EvilDoggie quickly sends your crafted <spoofed_data> message.
```

## How It Works

As previously mentioned, the spoofing attack waits until a certain condition (ID, Data) matches the configured arguments and immediately afterward, it sends a new message.

In the following example, we will run an attack on the message with ID `0x100` where the first data byte equals `0x02`, and we will send the same message with two additional data bytes `0x00` and `0x00`.
We do so with the following commands:

```
> spoofing_attack 0x100 0x2,0x0,0x0 0x2
> attack
```

Arguments:

* 0x100: CAN ID
* 0x2,0x0,0x0: data to spoof
* 0x2: only trigger spoof when the first byte is 0x2 

If we attach a logic analyzer, we could see something like this:

![Spoofing Attack Logic Analyzer](../../../res/spoofing_attack_la.png)

Here, we have 3 devices involved, and the channels are:

* **E TX**: evilDoggie TX
* **E RX**: evilDoggie RX
* **G0 TX**: Doggie 0 TX 
* **G1 TX**: Doggie 1 TX

We can see that **G0** sends the message, and just after the End of Frame, **E** sends the new message.


## Implementation

**TODO**: High level commands that compose the attack
