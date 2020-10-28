run_debug:
	qemu-system-riscv64 -s -S -machine virt -cpu rv64 -smp 1 -m 128M  -nographic -serial mon:stdio -bios none -kernel target/riscv64gc-unknown-none-elf/debug/tong_os

debug: tong_os
	riscv64-elf-gdb -ex "target remote localhost:1234" --symbols=target/riscv64gc-unknown-none-elf/debug/tong_os


