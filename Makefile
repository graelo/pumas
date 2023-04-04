.PHONY: all clean test

build-release:
	RUSTFLAGS="-Ctarget-cpu=native" cargo build --release
