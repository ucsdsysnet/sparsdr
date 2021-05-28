/**
 * Industrial I/O driver for configuring SparSDR compression on an Analog
 * Devices ADALM-PLUTO (Pluto) software-defined radio
 *
 * # Compiling
 *
 * ## Dependencies
 *
 * * Clone the Linux source from github.com/analogdevicesinc/linux/
 * * Check out the branch that matches the version running on your Pluto device
 *   (in one case, for kernel 4.14, the correct branch was `adi-4.14.0`)
 * * Compile Linux following [the instructions](https://wiki.analog.com/resources/tools-software/linux-build/generic/zynq)
 *
 * ## Actually compiling
 *
 * * Edit the Makefile, changing `KDIR` to the linux folder where you compiled
 *   the kernel
 * * Set up cross-compilation with these commands:
 *     * `export ARCH=arm`
 *     * `export CROSS_COMPILE=[the path to your ARM compiler binaries, including the arm-linux-gnueabi- prefix]`
 *       (example: `/media/samcrow/Profiterole/pluto/gcc-linaro-5.5.0-2017.10-x86_64_arm-linux-gnueabi/bin/arm-linux-gnueabi-`)
 * * Compile the kernel module by running `make`
 *
 * # Installing
 *
 * * Copy the `sparsdr_iio.ko` file onto the Pluto device using `scp`
 * * SSH to the Pluto device and run `insmod sparsdr_iio.ko`
 *
 * At this point, if you run `iio_info` on the Pluto, a `sparsdr` device with
 * several attributes should appear at the end of the output.
 *
 * The `sparsdr` IIO device can be controlled locally, but not from other
 * computers. Restart the IIO server so that it detects `sparsdr`:
 *
 * * `/etc/init.d/S23udc reload`
 *
 * This command will also restart the network interface, so you may need to
 * reconnect to the Pluto device from your computer. After that, `iio_info`
 * on the computer should be able to connect to the Pluto device and show
 * the SparSDR attributes.
 *
 * # Using
 *
 * TODO: Detailed documentation
 *
 * ## Examples
 *
 * * Read the FFT size from a host computer: `iio_attr -u ip:192.168.2.1 -d sparsdr fft_size`
 * * Set the FFT size from a host computer: `iio_attr -u ip:192.168.2.1 -d sparsdr fft_size 512`
 *
 */

#include <linux/init.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/slab.h>

#include <linux/device.h>
#include <linux/iio/iio.h>
#include <linux/iio/sysfs.h>
#include <asm/io.h>

MODULE_DESCRIPTION("IIO driver for configuring SparSDR compression");
MODULE_AUTHOR("Sam Crow");
MODULE_LICENSE("GPL");

/** Address in physical memory of the beginning of the registers */
static const size_t REGISTER_BASE = 0x7C440000;

/** Writes a 32-bit value to a register on the FPGA */
static void sparsdr_write_register(size_t index, u32 value) {
  size_t selected_register_physical = REGISTER_BASE + index * 4;
  void __iomem *register_virtual = ioremap(selected_register_physical, 4);
  iowrite32(value, register_virtual);
}

/** SparSDR IIO attributes */
enum sparsdr_dev_attr {
  SPARSDR_ENABLE_COMPRESSION,
  SPARSDR_RUN_FFT,
  SPARSDR_SEND_FFT_SAMPLES,
  SPARSDR_SEND_AVERAGE_SAMPLES,
  SPARSDR_FFT_SIZE,
  SPARSDR_FFT_SCALING,
  SPARSDR_BIN_MASK,
  SPARSDR_BIN_THRESHOLD,
  SPARSDR_AVERAGE_WEIGHT,
  SPARSDR_AVERAGE_INTERVAL,
};

/**
 * Private data used for this driver
 */
struct sparsdr_private_data {
  bool enable_compression;
  bool run_fft;
  bool send_fft_samples;
  bool send_average_samples;
  u32 fft_size;
  u32 fft_scaling;
  u8 average_weight;
  u32 average_interval;
};

/**
 * Reads an attribute from this driver to user space
 */
static ssize_t sparsdr_attr_read(struct device *dev,
                                 struct device_attribute *attr, char *buf) {
  struct iio_dev *indio_dev = dev_to_iio_dev(dev);
  struct iio_dev_attr *this_attr = to_iio_dev_attr(attr);
  struct sparsdr_private_data *data = iio_priv(indio_dev);

  int ret = 0;

  mutex_lock(&indio_dev->mlock);
  switch ((u32)this_attr->address) {
  case SPARSDR_ENABLE_COMPRESSION:
    ret = sprintf(buf, "%u\n", (unsigned int)data->enable_compression);
    break;
  case SPARSDR_RUN_FFT:
    ret = sprintf(buf, "%u\n", (unsigned int)data->run_fft);
    break;
  case SPARSDR_SEND_FFT_SAMPLES:
    ret = sprintf(buf, "%u\n", (unsigned int)data->send_fft_samples);
    break;
  case SPARSDR_SEND_AVERAGE_SAMPLES:
    ret = sprintf(buf, "%u\n", (unsigned int)data->send_average_samples);
    break;
  case SPARSDR_FFT_SIZE:
    ret = sprintf(buf, "%u\n", data->fft_size);
    break;
  case SPARSDR_FFT_SCALING:
    ret = sprintf(buf, "%u\n", data->fft_scaling);
    break;
  case SPARSDR_BIN_MASK:
    // This doesn't really have a readable value
    ret = sprintf(buf, "0\n");
    break;
  case SPARSDR_BIN_THRESHOLD:
    // This doesn't really have a readable value
    ret = sprintf(buf, "0\n");
    break;
  case SPARSDR_AVERAGE_WEIGHT:
    ret = sprintf(buf, "%u\n", (unsigned int)data->average_weight);
    break;
  case SPARSDR_AVERAGE_INTERVAL:
    ret = sprintf(buf, "%u\n", data->average_interval);
    break;
  default:
    ret = -EINVAL;
    break;
  }
  mutex_unlock(&indio_dev->mlock);

  return ret;
}

/**
 * Writes an attribute from user space to this driver
 */
static ssize_t sparsdr_attr_write(struct device *dev,
                                  struct device_attribute *attr,
                                  const char *buf, size_t len) {
  struct iio_dev *indio_dev = dev_to_iio_dev(dev);
  struct iio_dev_attr *this_attr = to_iio_dev_attr(attr);
  struct sparsdr_private_data *data = iio_priv(indio_dev);

  int ret = 0;

  mutex_lock(&indio_dev->mlock);
  switch ((u32)this_attr->address) {
  case SPARSDR_ENABLE_COMPRESSION:
    ret = strtobool(buf, &data->enable_compression);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(19, (u32)data->enable_compression);
    break;
  case SPARSDR_RUN_FFT:
    ret = strtobool(buf, &data->run_fft);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(17, (u32)data->run_fft);
    break;
  case SPARSDR_SEND_FFT_SAMPLES:
    ret = strtobool(buf, &data->send_fft_samples);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(15, (u32)data->send_fft_samples);
    break;
  case SPARSDR_SEND_AVERAGE_SAMPLES:
    ret = strtobool(buf, &data->send_average_samples);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(16, (u32)data->send_average_samples);
    break;
  case SPARSDR_FFT_SIZE:
    ret = kstrtou32(buf, 10, &data->fft_size);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(20, data->fft_size);
    break;
  case SPARSDR_FFT_SCALING:
    ret = kstrtou32(buf, 10, &data->fft_scaling);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(10, data->fft_scaling);
    break;
  case SPARSDR_BIN_MASK:
    // TODO
    ret = -ENOSYS;
    break;
  case SPARSDR_BIN_THRESHOLD:
    // TODO
    ret = -ENOSYS;
    break;
  case SPARSDR_AVERAGE_WEIGHT:
    ret = kstrtou8(buf, 10, &data->average_weight);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(13, (u32)data->average_weight);
    break;
  case SPARSDR_AVERAGE_INTERVAL:
    ret = kstrtou32(buf, 10, &data->average_interval);
    if (ret < 0) {
      break;
    }
    sparsdr_write_register(14, data->average_interval);
    break;
  default:
    ret = -EINVAL;
    break;
  }
  mutex_unlock(&indio_dev->mlock);

  return ret ? ret : len;
}

// Define attribute structs
static IIO_DEVICE_ATTR(enable_compression, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_ENABLE_COMPRESSION);
static IIO_DEVICE_ATTR(run_fft, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_RUN_FFT);
static IIO_DEVICE_ATTR(send_fft_samples, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_SEND_FFT_SAMPLES);
static IIO_DEVICE_ATTR(send_average_samples, S_IRUGO | S_IWUSR,
                       sparsdr_attr_read, sparsdr_attr_write,
                       SPARSDR_SEND_AVERAGE_SAMPLES);
static IIO_DEVICE_ATTR(fft_size, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_FFT_SIZE);
static IIO_DEVICE_ATTR(fft_scaling, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_FFT_SCALING);
static IIO_DEVICE_ATTR(bin_mask, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_BIN_MASK);
static IIO_DEVICE_ATTR(bin_threshold, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_BIN_THRESHOLD);
static IIO_DEVICE_ATTR(average_weight, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_AVERAGE_WEIGHT);
static IIO_DEVICE_ATTR(average_interval, S_IRUGO | S_IWUSR, sparsdr_attr_read,
                       sparsdr_attr_write, SPARSDR_AVERAGE_INTERVAL);

static struct attribute *sparsdr_attributes[] = {
    &iio_dev_attr_enable_compression.dev_attr.attr,
    &iio_dev_attr_run_fft.dev_attr.attr,
    &iio_dev_attr_send_fft_samples.dev_attr.attr,
    &iio_dev_attr_send_average_samples.dev_attr.attr,
    &iio_dev_attr_fft_size.dev_attr.attr,
    &iio_dev_attr_fft_scaling.dev_attr.attr,
    &iio_dev_attr_bin_mask.dev_attr.attr,
    &iio_dev_attr_bin_threshold.dev_attr.attr,
    &iio_dev_attr_average_weight.dev_attr.attr,
    &iio_dev_attr_average_interval.dev_attr.attr,
    NULL,
};

static const struct attribute_group sparsdr_attribute_group = {
    .attrs = sparsdr_attributes,
};

static const struct iio_info sparsdr_iio_info = {
    .attrs = &sparsdr_attribute_group,
};

/**
 * Global SparSDR IIO device
 */
static struct iio_dev *g_sparsdr_iio_dev = NULL;

static int sparsdr_init(void) {
  int register_status;

  pr_debug("SparSDR IIO loading\n");
  // Allocate memory for the device
  g_sparsdr_iio_dev = iio_device_alloc(sizeof(struct sparsdr_private_data));
  if (g_sparsdr_iio_dev == NULL) {
    return -ENOMEM;
  }
  // Fill in attributes
  g_sparsdr_iio_dev->name = "sparsdr";
  g_sparsdr_iio_dev->info = &sparsdr_iio_info;

  // Initialize default values in private data section
  struct sparsdr_private_data *data = iio_priv(g_sparsdr_iio_dev);
  // TODO: Check that these default values match the FPGA default values
  data->enable_compression = false;
  data->run_fft = false;
  data->send_fft_samples = false;
  data->send_average_samples = false;
  data->fft_size = 1024;
  data->fft_scaling = 0; // This is not correct
  data->average_weight = 200;
  data->average_interval = 1 << 14;

  register_status = iio_device_register(g_sparsdr_iio_dev);

  return register_status;
}

static void sparsdr_exit(void) {
  iio_device_unregister(g_sparsdr_iio_dev);
  iio_device_free(g_sparsdr_iio_dev);
  g_sparsdr_iio_dev = NULL;
  pr_debug("SparSDR IIO unloaded\n");
}

module_init(sparsdr_init);
module_exit(sparsdr_exit);
