# evilDoggie Get Started

## Evil Mode

evilDoggie has a different approach; it uses the serial interface to display a menu that lets the user configure attacks. The --omap lfcrlf and --imap lfcrlf arguments are used to match the control characters of the evilDoggie’s terminal. To exit from picocom, the keybinding is [ctrl + a, ctrl + x].

Now you should be in the evilDoggie console! If you press a key, you will see a banner and a prompt with ‘>’. You can type ‘help’ to see the available commands.

"GOOD" and "EVIL" is used to select the mode in which the board will boot. GOOD for Doggie and EVIL for evilDoggie. Note that you will need to reboot the board after switching modes if it is already powered.

## The Logic Behind evilDoggie Attacks

As an attacker you will connect Doggie or evilDoggie to the bus to complete the challenges.

## How It Is Physically Connected

As the attacks covered by evilDoggie target physical bus characteristics, there needs to be a real CAN bus where an attacker can connect. You can simulate a full CAN Bus without using a Doggie for each ECU! And then, as an attacker, you can connect your own evilDoggie and make an online attack, just like on a real car.
