# evilDoggie Introduction

evilDoggie is the offensive firmware variant of Doggie, created for advanced security research and low-level CAN Bus manipulation. It extends Doggie's capabilities with features for exploiting CAN protocol weaknesses, such as:

- Spoofing messages with bit-level precision.
- Injecting errors to trigger double receives or bus off states.
- Forcing dominant bits to override recessive ones at the physical layer.
- Bus takeover to silence other ECUs.
- Custom attack scripting for targeted vulnerabilities.

These features make evilDoggie a powerful tool for deeper research into how CAN networks can be broken and secured.

## Key features

### Force Feature

The Force feature is a physical-layer override mechanism that allows evilDoggie to assert dominant bits on the CAN Bus, bypassing standard arbitration and enabling attacks such as bus takeover or message alteration. By forcing recessive bits (logic 1) to dominant (logic 0), it ensures the attacker's frame prevails, even against higher-priority messages. For more details, refer to [force (software)](./force.md) and [force (hardware)](../../../hardware/force.md) section

### Custom Attack Feature

The custom attack feature provides a modular framework for chaining low-level primitives (e.g., bit injection, error frames, and delays) to craft bespoke attacks with bit-level precision. This enables researchers to experiment, prototype, and deploy novel exploits tailored to specific vulnerabilities in CAN networks. evilDoggie facilitates in-depth research and the creation of new attacks. Since all attacks are performed via bitbanging—directly manipulating the signal timing without relying on a dedicated CAN controller—they can theoretically be ported to any compromised microcontroller in a vehicle ECU, provided it has access to the TX and RX pins on the CAN Bus. This makes the techniques adaptable for real-world red teaming in automotive environments.

⚠ Important: evilDoggie is for research and training purposes only. Use it responsibly!

Project repository: [https://github.com/infobyte/doggie](https://github.com/infobyte/doggie)
