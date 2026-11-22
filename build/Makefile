# Resolve paths relative to this file, including when invoked with make -f.
MAKEFILE := $(abspath $(lastword $(MAKEFILE_LIST)))
ROOT := $(abspath $(dir $(MAKEFILE))/..)
SRC := $(ROOT)/src
BUILD ?= $(ROOT)/build
.DEFAULT_GOAL := all
.DELETE_ON_ERROR:
# Prefer PATH, then the ARM toolchain bundled with STM32CubeIDE.
ARM_GCC := $(or $(shell command -v arm-none-eabi-gcc 2>/dev/null),\
    $(firstword $(wildcard /opt/st/stm32cubeide_*/plugins/*/tools/bin/arm-none-eabi-gcc)),\
    arm-none-eabi-gcc)
CROSS ?= $(patsubst %-gcc,%,$(ARM_GCC))
CPU := cortex-a7
TARGET := armv7a-none-eabi

RUSTC ?= rustc
RUSTUP ?= rustup
CC := $(CROSS)-gcc
LD := $(CC)
OBJCOPY ?= $(CROSS)-objcopy
OBJDUMP ?= $(CROSS)-objdump
QEMU ?= qemu-system-arm

BOOT_SRC := $(SRC)/kernel/boot.S
KERNEL_SRC := $(SRC)/kernel/kernel.rs
LINKER := $(ROOT)/build/linker.ld

BOOT_O := $(BUILD)/boot.o
KERNEL_O := $(BUILD)/kernel.o
KERNEL_DEPS := $(BUILD)/kernel.d
ELF := $(BUILD)/kernel7.elf
IMG := $(BUILD)/kernel7.img

RUSTFLAGS := --edition 2024 \
    --crate-type lib \
    --target $(TARGET) \
    -C panic=abort \
    -C opt-level=2 \
    -C relocation-model=static \
    -C target-cpu=$(CPU)

all: $(IMG)

check-target:
	@$(RUSTUP) target list --installed | grep -qx '$(TARGET)' || \
		{ echo "missing Rust target '$(TARGET)'"; \
		  echo "hint: run 'rustup target add $(TARGET)'"; \
		  exit 1; }

check-tools:
	@command -v $(CC) >/dev/null 2>&1 || \
		{ echo "missing cross compiler '$(CC)'"; \
		  echo "hint: install the GNU Arm embedded toolchain or override CC=..."; \
		  exit 1; }
	@command -v $(OBJCOPY) >/dev/null 2>&1 || \
		{ echo "missing objcopy tool '$(OBJCOPY)'"; \
		  echo "hint: install the GNU Arm embedded toolchain or override OBJCOPY=/path/to/objcopy"; \
		  exit 1; }

check-runner:
	@command -v $(QEMU) >/dev/null 2>&1 || \
		{ echo "missing emulator '$(QEMU)'"; \
		  echo "hint: install qemu-system-arm or override QEMU=..."; \
		  exit 1; }

$(BUILD):
	mkdir -p $@

$(BOOT_O): $(BOOT_SRC) $(MAKEFILE) | check-tools $(BUILD)
	$(CC) -c -mcpu=$(CPU) -ffreestanding -o $@ $<

$(KERNEL_O): $(KERNEL_SRC) $(MAKEFILE) | check-target $(BUILD)
	$(RUSTC) $(RUSTFLAGS) --emit=obj=$@,dep-info=$(KERNEL_DEPS) $<

$(ELF): $(BOOT_O) $(KERNEL_O) $(LINKER) $(MAKEFILE) | check-tools
	$(LD) -nostdlib -Wl,--build-id=none -T $(LINKER) -o $@ $(BOOT_O) $(KERNEL_O)

$(IMG): $(ELF) $(MAKEFILE) | check-tools
	$(OBJCOPY) -O binary $< $@

run: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -kernel $(ELF)

run-img: check-runner $(IMG)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -device loader,file=$(IMG),addr=0x8000,cpu-num=0,force-raw=on

debug: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -kernel $(ELF) -S -s

dump: $(ELF)
	$(OBJDUMP) -d $<

clean:
	rm -f $(BOOT_O) $(KERNEL_O) $(KERNEL_DEPS) $(ELF) $(IMG)

.PHONY: all check-target check-tools check-runner run run-img debug dump clean

# rustc records all modules used by the kernel, including shared code.
-include $(KERNEL_DEPS)
