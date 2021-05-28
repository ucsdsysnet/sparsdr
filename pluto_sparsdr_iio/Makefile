# To use:
#
# export ARCH=arm
# export CROSS_COMPILE=/media/samcrow/Profiterole/pluto/gcc-linaro-5.5.0-2017.10-x86_64_arm-linux-gnueabi/bin/arm-linux-gnueabi-
# make

# Change this to the directory where you compiled Linux
KDIR = /media/samcrow/Profiterole/pluto/linux

kbuild:
	$(MAKE) -C $(KDIR) M=`pwd`

clean:
	$(MAKE) -C $(KDIR) M=`pwd` clean
