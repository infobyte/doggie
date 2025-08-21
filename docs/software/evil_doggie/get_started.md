# evilDoggie Get Started

## Switching to Evil Mode (Faraday's Doggie)

evilDoggie operates in "EVIL" mode, which is selected via a physical switch on the hardware. To boot in EVIL mode for evilDoggie (as opposed to "GOOD" mode for standard Doggie):

- Set the switch to "EVIL".
- Reset the board or reconnect it to power.
- Note: If the board is already powered, reboot it after switching modes to apply the change.

This mode enables the offensive features, using the serial interface to display an interactive menu for configuring and launching attacks.

## Accessing the Serial Console

evilDoggie uses the serial interface to provide an interactive console menu for attack configuration. Unlike standard Doggie, which bridges CAN Bus via slcan for tools like SocketCAN, evilDoggie focuses on direct manipulation through this menu.

To connect to the serial console, use a terminal emulator like picocom. Assuming you're on Ubuntu, install it with:

```bash
sudo apt install -y picocom
```

Run picocom with the corresponding USB device (replace `/dev/ttyUSBx` with your actual device, e.g., `/dev/ttyUSB0` or `/dev/ttyACM0`):

```bash
picocom -q -b 921600 /dev/ttyUSBx --omap lfcrlf --imap lfcrlf
```

- `-q`: Quiet mode (suppresses non-essential output).
- `-b 921600`: Sets the baud rate to 921600, which is required for evilDoggie's serial communication.
- `--omap lfcrlf` and `--imap lfcrlf`: Map line feed (LF) to carriage return line feed (CRLF) for output and input, matching evilDoggie's control characters to ensure proper display and input handling.

To exit picocom, press `[Ctrl + A]` followed by `[Ctrl + X]`.

Once connected, press any key to display the banner and the prompt `>`. This enters the evilDoggie console.

- Type `help` to see information about the current menu and available commands.
- Type `help [command]` (e.g., `help spoofing_attack`) for details on a specific command.

## The Logic Behind evilDoggie Attacks

evilDoggie allows you to build an "attack plan," which is a sequence of attacks executed in order. The main menu provides predefined attacks (e.g., spoofing, bus off) that you can add to the plan. For more advanced control, use the `custom_attack` submenu to chain low-level attack primitives (e.g., bit-level manipulations) into custom attacks.

### Building an Attack Plan
- Use commands like `spoofing_attack` to add predefined attacks to the plan.
- Enter `custom_attack` to build custom ones:
    - Chain primitives (e.g., wait for a frame, inject error, force dominant bit).
    - Use `save` to add the custom attack to the main plan.
    - Use `exit` to return to the main menu.
- Edit the plan with `list` (view plan), `delete` (remove item), and `move` (reorder).
- Launch the plan with `attack [count]` (e.g., `attack 20` to run 20 times).

As an attacker, connect evilDoggie to the CAN Bus to execute these attacks, targeting physical bus characteristics for real-time manipulation, just like in a vehicle.

## How It Is Physically Connected

The attacks in evilDoggie exploit physical CAN Bus properties, requiring a connection to a real CAN Bus. In workshops or simulations, you can emulate multiple ECUs on a single bus without needing a Doggie per ECU. Then, connect your evilDoggie as the attack device for online exploitation, mimicking real-car scenarios.

For hardware connections, refer to the specific microcontroller guides in the Hardware/DIY section (e.g., ESP32, RP2040, Bluepill). Ensure the device is in EVIL mode and connected via USB or UART for serial access.
