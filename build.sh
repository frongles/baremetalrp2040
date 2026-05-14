
arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/baremetal target/thumbv6m-none-eabi/release/baremetal.bin
/usr/local/bin/compute_crc ./target/thumbv6m-none-eabi/release/baremetal.bin
 
