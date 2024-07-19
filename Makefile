.PHONY: all release debug

all:
	@cargo build
	@llvm-objcopy -O binary target/aarch64-unknown-none/debug/rustos kernel8.img 

release:
	@cargo build --release
	@llvm-objcopy -O binary target/aarch64-unknown-none/debug/rustos kernel8.img 
