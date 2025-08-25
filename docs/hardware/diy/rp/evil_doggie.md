# evilDoggie #

evilDoggie supports only the `uart` feature, so it can be built and flashed using the following command:

```
DEFMT_LOG=off cargo run --release --bin evil_doggie --no-default-features --features=uart
```

## **Supported Configurations**
The RP2040 implementation supports only one configuration where the GPIOs are connected directly to a standard CAN Bus driver or the [evilDoggie custom driver](../../force.md).

To connect a standard driver we will only need two GPIOs (CAN TX, CAN RX), but for the custom driver, we will need one more GPIO (CAN FORCE).

__Connections__:

| Function  | RP2040 |   Driver   | Custom Driver |
| :-------: | :----: |:--------: | :-----------: |
| Vcc       |  3v3   |    VCC     |      VCC      |
| GND       |  GND   |    GND     |      GND      |
| CAN TX    |   20   |    TX      |      TX       |
| CAN RX    |   21   |    RX      |      RX       |
| CAN FORCE |   22   |    -       |     FORCE     |
