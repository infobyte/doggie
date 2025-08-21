# Challenges

## Getting the VIN number
In this first challenge, your goal is to retrieve the car’s VIN number by sending a diagnostic request over the CAN Bus.

Modern cars store the VIN (Vehicle Identification Number) inside an ECU, and diagnostic tools can read it by sending a special request. You have to do the same, but using Doggie to send the request and listen for the response, just like a mechanic (or an attacker!) would.

### Hints
**What is the VIN?**

The Vehicle Identification Number (VIN) is a unique 17-character identifier every car has. In real life, mechanics and dealerships use diagnostic tools to read it through the OBD2 port by sending a standardized request.

**What protocol is used to retrieve the VIN?**

While normal CAN messages are short (up to 8 bytes), some diagnostics need to exchange longer data, like the VIN. To achieve this, the automotive world utilizes UDS (Unified Diagnostic Services, ISO 14229), which runs over ISO-TP (ISO 15765-2), a protocol designed to send multi-frame messages over CAN.

With ISO-TP, the VIN ECU can:

- Receive a diagnostic request from the tester (your tool)
- Send back a multi-frame response containing the VIN

**Which ECUs are involved?**
In our simulator, there’s a special VIN ECU that:

- Listens for ISO-TP requests sent to CAN ID **0x7E8**
- Replies with the VIN using ISO-TP

You act as the tester, using CAN ID **0x7DF**.

**Linux tools you’ll use:**

To interact over ISO-TP, you can use two powerful tools from can-utils:

- **isotpsend**: to send a diagnostic request
- **isotprecv**: to listen for the diagnostic response

They set up a virtual ISO-TP link over the physical CAN interface (your Doggie).

**Diagnostic request details:**

The UDS protocol has a series of services and parameter IDs. To get the VIN, you can use:

- **Service 09**: Request Vehicle Information
- **PID 02**: VIN

So, the request payload becomes: 09 02 (hex). In a real scan tool, this would be built automatically—but here you’ll craft and send it manually to understand what happens behind the scenes.

<figure align="center">
    <img src="res/workshop_vin_msgs.png" alt="VIN Messages">
</figure>

### Solution
**Start listening for the VIN ECU response**

Open a terminal and run:

`isotprecv -s 7DF -d 7E8 -p 8:8 -l doggie`

Where the arguments have the following meaning:

- -s 7DF: source (tester’s) CAN ID
- -d 7E8: destination (VIN ECU’s) CAN ID
- -p 8:8: padding config (common ISO-TP setting)
- -l: listen mode (prints raw response)

**Send the diagnostic request**

In another terminal, send the request:

`echo "09 02" | isotpsend -s 7DF -d 7E8 -p 8:8 doggie`

This sends a VIN request to the VIN ECU.

**See the response**

The isotprecv terminal should print something like:

`49 02 01 57 58 30 58 58 58 58 58 58 58 58 58 58 58`

Example:
<figure align="center">
    <img src="res/workshop_vin_example.png" alt="VIN Example">
</figure>
