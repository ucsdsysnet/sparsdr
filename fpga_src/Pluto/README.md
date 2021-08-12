# SparSDR patches for Pluto

To compile a SparSDR Pluto image, start with these upstream repositories:

* [Buildroot](https://github.com/analogdevicesinc/buildroot/)
* [HDL](https://github.com/analogdevicesinc/hdl)

Follow these steps:

* Check out version `a9818ab69cbe2dbbe6c3bfd1ea4634ea17699a46` of Buildroot
* Check out version `061d024d596ef84c6a819854bf2472e6b43a2d5d` of HDL
* Apply the two patches `1_Revert_to_Rev_B.patch` and `2_SparSDR_1_for_Pluto.patch` to the HDL repository
* Apply `3_SparSDR_1_iio_init.patch` to the Buildroot repository
* Follow the standard instructions to generate an image.
