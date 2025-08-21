# Force Feature - Software Level

The Force feature in evilDoggie is a powerful capability designed to manipulate CAN Bus communication at a low level, enabling physical-layer overrides. This feature allows an attacker to force recessive bits (logic 1) to dominant bits (logic 0) in real-time CAN frames, effectively overriding legitimate ECU transmissions. It is particularly useful for scenarios where precise control over bus arbitration or data integrity is required.

evilDoggie’s force mode (also called dominant-override) can:

- Take control of the bus.
- Force recessive bits even when other ECUs try to send dominant bits.
- This lets us inject a message in a way that other ECUs cana't.

This feature is enabled with a flag in the `send_raw` and `send_msg` primitives within the `custom_attack` submenu. This flag makes the `send_*` command force all the recessive bits that the command will send into the bus.

## Software Implementation

- **Activation**: The Force feature is enabled by appending the a--`force` flag to the `send_raw` or `send_msg` commands. For example:
    - `send_raw 1010101 --force` sends a raw binary data using the force mechanism.
    - `send_msg 0x200 0x01,0x02 --force` sends a message with ID `0x200` and data `0x01,0x02` with forced transmission.
- **Mechanism**: 
    - When the `--force` flag is set, in every recessive bit, the `force` GPIO will be pulled high in order to force it. See [the hardware force section](/hardware/force) for more ingormation.

  This software-level control enhances the flexibility of custom attacks, allowing evilDoggie to manipulate critical CAN Bus functions.

