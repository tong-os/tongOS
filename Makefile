PHONY:=mount umount clean

mount: | hdd hdd.dsk
	sudo losetup /dev/loop0 hdd.dsk
	sudo mount /dev/loop0 ./hdd

hdd:
	mkdir -p hdd

hdd.dsk:
	fallocate -l 32M hdd.dsk
	sudo losetup /dev/loop0 hdd.dsk
	sudo mkfs.minix -3 /dev/loop0
	sudo losetup -d /dev/loop0

umount:
ifeq ($(wildcard hdd), hdd)
	sudo umount -q /dev/loop0
	sudo losetup -d /dev/loop0
	rm -rf hdd
endif

clean: umount
	rm -f hdd.dsk

tong_os: hdd.dsk
	cargo build

run_debug:
	qemu-system-riscv64 -s -S -machine virt -cpu rv64 -smp 1 -m 128M -drive if=none,format=raw,file=hdd.dsk,id=foo -device virtio-blk-device,scsi=off,drive=foo -nographic -serial mon:stdio -bios none -device virtio-rng-device -device virtio-gpu-device -device virtio-net-device -device virtio-tablet-device -device virtio-keyboard-device -kernel target/riscv64gc-unknown-none-elf/debug/tong_os

debug: tong_os
	riscv64-elf-gdb -ex "target remote localhost:1234" --symbols=target/riscv64gc-unknown-none-elf/debug/tong_os


