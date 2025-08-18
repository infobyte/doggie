# (evil)Doggie dual boot
The esp32 bootloader allows to boot different apps depending on a GPIO input state. The bootloader image in this directory leverages this to switch between doggie and evil_doggie firmware images. It reads from pin 35 to decide witch image to boot.

To flash an esp32 with this bootloader and both firmwares follow these steps:
1. Build both doggie and evil_doggie firmware images.
2. Run: `espflash flash --flash-size 8mb --partition-table bootloader/partitions.csv --bootloader bootloader/bootloader.bin --target-app-partition test target/xtensa-esp32-none-elf/release/doggie`
3. Run: `espflash flash --flash-size 8mb --partition-table bootloader/partitions.csv --bootloader bootloader/bootloader.bin --target-app-partition factory target/xtensa-esp32-none-elf/release/evil_doggie`
4. You should be able to use a switch connected to GPIO 35 to select the boot image.
