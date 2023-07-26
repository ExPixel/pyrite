TARGET		:= bios
BUILD		:= build
SOURCE		:= source
INCLUDE		:= include

ASFLAGS     := -mcpu=arm7tdmi -march=armv4t -mthumb-interwork
CFLAGS      := -I$(INCLUDE) -mcpu=arm7tdmi -march=armv4t -mthumb-interwork -ffreestanding -static -nostdlib -c -Os
AS          := $(DEVKITARM)/bin/arm-none-eabi-as
CC          := $(DEVKITARM)/bin/arm-none-eabi-gcc
LD          := $(DEVKITARM)/bin/arm-none-eabi-ld
OBJCOPY     := $(DEVKITARM)/bin/arm-none-eabi-objcopy
OBJDUMP     := $(DEVKITARM)/bin/arm-none-eabi-objdump

AS_SOURCES  := $(wildcard $(SOURCE)/*.s)
AS_OBJECTS  := $(patsubst $(SOURCE)/%.s,$(BUILD)/%.o,$(AS_SOURCES))

C_SOURCES   := $(wildcard $(SOURCE)/*.c)
C_OBJECTS   := $(patsubst $(SOURCE)%.c,$(BUILD)%.co,$(C_SOURCES))

OBJECTS     := $(AS_OBJECTS) $(C_OBJECTS)

.PHONY: $(BUILD)

all: $(BUILD)

$(BUILD)/%.o: source/%.s
	$(AS) $(ASFLAGS) -o $@ $<
$(BUILD)/%.co: source/%.c
	$(CC) $(CFLAGS) -o $@ $<

$(BUILD)/$(TARGET).elf: $(OBJECTS)
	$(LD) --script source/$(TARGET).ld -o $@ $^

$(BUILD)/$(TARGET).bin: $(BUILD)/$(TARGET).elf
	$(OBJCOPY) -O binary $< $@

$(BUILD):
	@[ -d $@ ] || mkdir -p $@
	@$(MAKE) $(BUILD)/$(TARGET).bin

.PHONY: clear
clear:
	rm -f	$(BUILD)/$(TARGET).elf
	rm -f	$(OBJECTS)
	rm -rf	$(BUILD)

.PHONY: clean
clean:	clear
	rm -f build/$(TARGET).bin