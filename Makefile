.PHONY: all clean
all: bios mode3-test
clean: clean-bios clean-mode3-test

.PHONY: bios clean-bios
bios:
	cd gba-programs/custom-bios && make && cp build/bios.bin ../../roms/custom/custom-bios.bin
clean-bios:
	cd gba-programs/custom-bios && make clean

.PHONY: mode3-test clean-mode3-test
mode3-test:
	cd gba-programs/mode3-test && make && cp mode3-test.gba ../../roms/custom/mode3-test.gba
clean-mode3-test:
	cd gba-programs/mode3-test && make clean
