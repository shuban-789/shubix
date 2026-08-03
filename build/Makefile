ROOT := ..
SRC := $(ROOT)/src
BUILD := .
CROSS := arm-none-eabi
CPU := cortex-a7
TARGET := armv7a-none-eabi
HOST_TRIPLE := $(shell rustc -vV | sed -n 's/^host: //p')
SYSROOT := $(shell rustc --print sysroot)
RUST_BIN := $(SYSROOT)/lib/rustlib/$(HOST_TRIPLE)/bin

RUSTC ?= rustc
RUSTUP ?= rustup
CC := $(CROSS)-gcc
LD := $(CC)
OBJCOPY ?= $(RUST_BIN)/rust-objcopy
OBJDUMP ?= objdump
QEMU ?= qemu-system-arm

BOOT_SRC := $(SRC)/boot.S
KERNEL_SRC := $(SRC)/main.rs
LINKER := $(BUILD)/linker.ld

BOOT_O := $(BUILD)/boot.o
KERNEL_O := $(BUILD)/kernel.o
ELF := $(BUILD)/kernel7.elf
IMG := $(BUILD)/kernel7.img

RUSTFLAGS := --edition 2024 \
    --crate-type lib \
    --target $(TARGET) \
    -C panic=abort \
    -C relocation-model=static \
    -C target-cpu=$(CPU)

all: check-target check-tools $(IMG)

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
	@test -x "$(OBJCOPY)" || \
		{ echo "missing objcopy tool '$(OBJCOPY)'"; \
		  echo "hint: use Rust's bundled rust-objcopy or override OBJCOPY=/path/to/objcopy"; \
		  exit 1; }

check-runner:
	@command -v $(QEMU) >/dev/null 2>&1 || \
		{ echo "missing emulator '$(QEMU)'"; \
		  echo "hint: install qemu-system-arm or override QEMU=..."; \
		  exit 1; }

$(BOOT_O): $(BOOT_SRC)
	$(CC) -c -mcpu=$(CPU) -ffreestanding -o $@ $<

$(KERNEL_O): $(KERNEL_SRC)
	$(RUSTC) $(RUSTFLAGS) --emit=obj -o $@ $<

$(ELF): $(BOOT_O) $(KERNEL_O) $(LINKER)
	$(LD) -nostdlib -Wl,--build-id=none -T $(LINKER) -o $@ $(BOOT_O) $(KERNEL_O)

$(IMG): $(ELF)
	$(OBJCOPY) -O binary $< $@

run: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -kernel $<

run-img: check-runner $(IMG)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -kernel $<

debug: check-runner $(ELF)
	$(QEMU) -M raspi2b -cpu $(CPU) -nographic -kernel $< -S -s

dump: $(ELF)
	$(OBJDUMP) -d $<

clean:
	rm -f $(BOOT_O) $(KERNEL_O) $(ELF) $(IMG)

.PHONY: all check-target check-tools check-runner run run-img debug dump clean
