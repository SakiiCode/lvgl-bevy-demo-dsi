esptools tool --chip esp32p4 elf2image --output $1.img --flash_size 32MB $1
espflash write-bin --chip esp32p4 --baud 460800 --no-stub --monitor 0x10000 $1.img