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