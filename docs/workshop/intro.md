# evilDoggie Workshop Introduction

## Workshop Overview
This workshop is designed to introduce participants to Doggie and its offensive variant, evilDoggie. Rather than teaching how a real car works, the workshop focuses on using Doggie and evilDoggie to analyze and manipulate a simulated CAN Bus environment specially built for training. Through several progressive challenges, participants will learn how Doggie integrates with standard tools and how, using evilDoggie, an attacker could take advantage of this protocol. For more information about CAN protocol and CAN hacking, check the appendix at the end of this guide.

## Content Outline
- Introduction
- Doggie and evilDoggie
  - What is Doggie?
  - What is evilDoggie?
- Get Started
  - Good Mode
    - Install tools
    - Create the CAN interface
    - Bring up the interface
    - Start sniffing with candump
  - Evil Mode
    - The logic behind evilDoggie attacks
    - How this workshop works
    - How it is physically connected
- Challenges
  - 1. VIN Spoofing (details in full guide)
  - Speed Spoofing: Use spoofing_attack to set speed to 0.
  - Door Unlock: Handle bus contention with higher priority messages.
  - Airbag Disable: Use double receive attack by injecting errors in EOF field.
  - Bus Off Attack: Force target ECU offline by injecting errors.
  - Engine Start without Key: Use force mode to override key status at physical level.

## Detailed Challenges

### Challenge: Speed Spoofing
The car’s speed is 0 km/h. Instead of just sniffing or sending normal messages, here you’ll need to use evilDoggie, the offensive firmware variant of Doggie. You’ll apply a classic attack known as spoofing, where you send fake messages with speed = 0.

evilDoggie provides a special spoofing_attack command:
> spoofing_attack 0x100 0x2,0x0,0x0 0x2

- 0x100: CAN ID
- 0x2,0x0,0x0: data (speed = 0)
- 0x2: mask (spoof only other messages on the same ID)

Run the attack multiple times with > attack 20.

Observe: The accelerator goes to 0% as the CC ECU now knows the actual speed.

### Challenge: Door Unlock
Here’s how to build and run this attack step by step with evilDoggie.

First, open evilDoggie in your terminal.

The first part of the attack: Notice how a GoodDoggie tries to send a message with higher priority, just as the evilDoggie is trying to send a door unlock message. This results in a bus error.

A successful one looks like this: In this case, there is no contention for the bus. The evilDoggie sends the door unlock message successfully, and is ACK’ed by the GoodDoggies.

This can be avoided using the bus takeover feature of evilDoggie. By taking full control of the bus and silencing other ECUs, the attacker can ensure that their messages go through.

### Challenge: Airbag Disable
Here’s how to do this step by step with evilDoggie.

To set up a double receive attack, use: Toggle the airbag button on the simulator UI.

If the attack succeeds:
- evilDoggie’s console will log that the double receive attack was launched.
- The Instrument Cluster will show the airbag disabled.

Observe the result in the logic analyzer: evilDoggie injects an error just after the 6th bit of the EOF field in the CAN frame corresponding to the Airbag.

### Challenge: Bus Off Attack
Here’s how to do this attack using evilDoggie.

> bus_off_attack 0x105 50

evilDoggie will wait for messages with ID 0x105 (ABS heartbeat) and inject errors, effectively silencing the target ECU for a while.

Observe: The evilDoggie injecting errors after each ABS heartbeat message causing the ABS ECU to go silent for a period.

### Challenge: Engine Start without Key
The engine can’t start unless the Immo ECU reports the key is present. You’ll use a unique feature of evilDoggie called force to override the messages that report the key status at the physical level.

evilDoggie’s force mode (also called dominant-override) can:
- Take control of the bus.
- Force recessive bits to dominant at the physical level.

This keeps the Immo ECU from sending “key not present” messages and allows evilDoggie to send forged “key is present” messages, followed by a start command.

Solution: We’ll build a custom attack in evilDoggie using the force feature.

Enter the custom attack menu:
> custom_attack

First, wait until the attack succeeded.

Congratulations—you’ve completed the final and most advanced challenge, using evilDoggie’s physical-layer force capability to break the car’s security and start the engine without a key!

Observe in the logic analyzer: evilDoggie is forcing the KeyMessage indicating that the key is present. Depending on the timing you might see the RX line does not match the data its trying to send, it enters into an error state. Finally, evilDoggie sends the message to start the car.

## Appendix: CAN Protocol
The CAN protocol is always evolving. You can build your own DIY Doggie by combining parts you already have! For more information on this, check the appendix on CAN protocol at the end of this guide. And that’s exactly why we’ll use tools like Doggie and evilDoggie to see how these protocol details can be turned into practical attacks.
