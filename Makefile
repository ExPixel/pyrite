.PHONY: all bios clean-bios

all: bios
clean: clean-bios

bios:
	cd gba-programs/custom-bios && make && cp build/bios.bin ../../roms/misc/custom-bios.bin
clean-bios:
	cd gba-programs/custom-bios && make clean
