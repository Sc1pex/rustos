.ONESHELL: all debug chainloader
.PHONY: all debug chainloader

all:
	@RUSTFLAGS="-C link-arg=--script=kernel/link.ld" cargo build -p kernel --release
	@llvm-objcopy -O binary target/aarch64-unknown-none/release/kernel kernel8.img 

debug:
	@RUSTFLAGS="-C link-arg=--script=kernel/link.ld" cargo build -p kernel --features debug_wait
	@llvm-objcopy -O binary target/aarch64-unknown-none/debug/kernel kernel8.img 

chainloader:
	@RUSTFLAGS="-C link-arg=--script=chainloader/link.ld" cargo build -p chainloader --release
	@llvm-objcopy -O binary target/aarch64-unknown-none/release/chainloader kernel8.img 

chainloader-debug:
	@RUSTFLAGS="-C link-arg=--script=chainloader/link.ld" cargo build -p chainloader
	@llvm-objcopy -O binary target/aarch64-unknown-none/debug/chainloader kernel8.img 

push: all
	@./serial/serial kernel8.img

push-debug: debug
	@./serial/serial kernel8.img
