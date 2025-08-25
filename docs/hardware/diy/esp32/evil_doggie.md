# evilDoggie #

evilDoggie supports only the `ble` features. And default, `ble` is enabled.

For example:
```
# evilDoggie on ESP32c3 with BLE
DEFMT_LOG=off cargo esp32c3 --bin evil_doggie

# evilDoggie on ESP32 without BLE
DEFMT_LOG=off cargo esp32 --bin evil_doggie --disable-default-features
```

TODO: Hardware config