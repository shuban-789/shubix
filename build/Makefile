MAKEFILE := $(abspath $(lastword $(MAKEFILE_LIST)))
ROOT := $(abspath $(dir $(MAKEFILE))/..)
SRC := $(ROOT)/src
BUILD ?= $(ROOT)/build
.DEFAULT_GOAL := all
.DELETE_ON_ERROR:

ARM_GCC := $(or $(shell command -v arm-none-eabi-gcc 2>/dev/null),\
    $(firstword $(wildcard /opt/st/stm32cubeide_*/plugins/*/tools/bin/arm-none-eabi-gcc)),\
    arm-none-eabi-gcc)
CROSS ?= $(patsubst %-gcc,%,$(ARM_GCC))
CPU := cortex-a7
TARGET := armv7a-none-eabi
RUSTC ?= rustc
RUSTUP ?= rustup
CC := $(CROSS)-gcc
OBJCOPY ?= $(CROSS)-objcopy
OBJDUMP ?= $(CROSS)-objdump
QEMU ?= qemu-system-arm

BOOT_SRC := $(SRC)/kernel/boot.S
KERNEL_SRC := $(SRC)/kernel/kernel.rs
LINKER := $(ROOT)/build/linker.ld
BOOT_O := $(BUILD)/boot.o
KERNEL_LIB := $(BUILD)/libkernel.a
KERNEL_DEPS := $(BUILD)/kernel.d
ELF := $(BUILD)/kernel7.elf
IMG := $(BUILD)/kernel7.img

RUSTFLAGS := --edition 2024 --crate-name shubix --crate-type staticlib \
    --target $(TARGET) -C panic=abort -C opt-level=2 -C debuginfo=2 \
    -C relocation-model=static -C target-cpu=$(CPU)

all: $(IMG)

check-target:
	@$(RUSTUP) target list --installed | grep -qx '$(TARGET)' || \
		{ echo "Install the Rust target with: rustup target add $(TARGET)"; exit 1; }

check-tools:
	@command -v $(CC) >/dev/null 2>&1 || { echo "Missing ARM compiler: $(CC)"; exit 1; }
	@command -v $(OBJCOPY) >/dev/null 2>&1 || { echo "Missing ARM objcopy: $(OBJCOPY)"; exit 1; }

check-runner:
	@command -v $(QEMU) >/dev/null 2>&1 || { echo "Missing QEMU: $(QEMU)"; exit 1; }

$(BUILD):
	mkdir -p $@

$(BOOT_O): $(BOOT_SRC) $(MAKEFILE) | check-tools $(BUILD)
	$(CC) -c -mcpu=$(CPU) -ffreestanding -g -o $@ $<

$(KERNEL_LIB): $(KERNEL_SRC) $(MAKEFILE) | check-target $(BUILD)
	$(RUSTC) $(RUSTFLAGS) --emit=link,dep-info=$(KERNEL_DEPS) -o $@ $<

$(ELF): $(BOOT_O) $(KERNEL_LIB) $(LINKER) $(MAKEFILE) | check-tools
	$(CC) -mcpu=$(CPU) -nostdlib -Wl,--build-id=none,--gc-sections,-z,noexecstack -T $(LINKER) -o $@ $(BOOT_O) $(KERNEL_LIB) -lgcc

$(IMG): $(ELF) $(MAKEFILE) | check-tools
	$(OBJCOPY) -O binary $< $@

run: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -display none -serial mon:stdio -kernel $(ELF)

run-display: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -serial mon:stdio -kernel $(ELF)

run-img: check-runner $(IMG)
	$(QEMU) -M raspi2b -cpu $(CPU) -display none -serial mon:stdio -device loader,file=$(IMG),addr=0x8000,cpu-num=0,force-raw=on

debug: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -display none -serial mon:stdio -kernel $(ELF) -S -s

dump: $(ELF)
	$(OBJDUMP) -d $<

clean:
	rm -f $(BOOT_O) $(KERNEL_LIB) $(KERNEL_DEPS) $(ELF) $(IMG)

.PHONY: all check-target check-tools check-runner run run-display run-img debug dump clean
-include $(KERNEL_DEPS)
