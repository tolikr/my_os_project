# Target architecture and toolchain
TARGET := riscv64gc-unknown-none-elf
AS := riscv64-unknown-elf-as
CC := riscv64-unknown-elf-gcc
CFLAGS := -Wall -ggdb -ffreestanding -nostdlib -I./include
LD := rust-lld
QEMU := qemu-system-riscv64
VERSION := debug

# File paths
SRC_DIR := src
ASM_DIR := $(SRC_DIR)/asm
BOOT_ASM := $(ASM_DIR)/boot.s
TRAP_ASM := $(ASM_DIR)/trap.s
KERNEL_RS := $(SRC_DIR)/lib.rs
LINKER_SCRIPT := linker.ld

# Output filenames
BOOT_OBJ := $(ASM_DIR)/boot.o
TRAP_OBJ := $(ASM_DIR)/trap.o
CRATE_NAME := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')
KERNEL_OBJ := target/$(TARGET)/$(VERSION)/lib$(CRATE_NAME).a
KERNEL_ELF := kernel.elf

# QEMU options
QEMU_FLAGS := -machine virt -cpu max -nographic -kernel $(KERNEL_ELF)

# Build the kernel
all: $(KERNEL_ELF)

# Assemble the boot.s to boot.o
$(BOOT_OBJ): $(BOOT_ASM)
	$(AS) $< -o $@

# Assemble the trap.s to trap.o
$(TRAP_OBJ): $(TRAP_ASM)
	$(AS) $< -o $@

# Compile the Rust kernel to an object file
$(KERNEL_OBJ): $(KERNEL_RS)
	cargo build --target $(TARGET)

# Link the kernel object and boot object into an ELF
$(KERNEL_ELF): $(BOOT_OBJ) $(TRAP_OBJ) $(KERNEL_OBJ) $(LINKER_SCRIPT)
	$(CC) -nostdlib -no-pie -T $(LINKER_SCRIPT) -o $@ $(BOOT_OBJ) $(TRAP_OBJ) $(KERNEL_OBJ)

# Run the kernel with QEMU
run: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS)

# Clean up build artifacts
clean:
	cargo clean
	rm -rf target
	rm -f $(BOOT_OBJ) $(TRAP_OBJ) $(KERNEL_ELF)

.PHONY: all run clean