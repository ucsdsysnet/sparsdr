// SPDX-License-Identifier: GPL-2.0-or-later
/**
 * Industrial I/O driver for configuring SparSDR compression on an Analog
 * Devices ADALM-PLUTO (Pluto) software-defined radio
 *
 * Copyright (C) 2021 The Regents of the University of California
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; either version 2 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program; if not, write to the Free Software Foundation, Inc., 51
 * Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
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
static void sparsdr_write_register(size_t index, u32 value)
{
	size_t selected_register_physical = REGISTER_BASE + index * 4;
	void __iomem *register_virtual = ioremap(selected_register_physical, 4);
	iowrite32(value, register_virtual);
}

/** Registes available on the FPGA */
enum sparsdr_register {
	REGISTER_FFT_SCALING = 10,
	REGISTER_THRESHOLD_BIN_NUMBER = 11,
	REGISTER_BIN_MASK = 12,
	REGISTER_AVERAGE_WEIGHT = 13,
	REGISTER_AVERAGE_INTERVAL = 14,
	REGISTER_FFT_SEND = 15,
	REGISTER_AVERAGE_SEND = 16,
	REGISTER_RUN_FFT = 17,
	REGISTER_WINDOW_VAL = 18,
	REGISTER_ENABLE_COMPRESSION = 19,
	REGISTER_FFT_SIZE = 20,
	REGISTER_THRESHOLD_VALUE = 21,
};

/** SparSDR IIO attributes */
enum sparsdr_dev_attr {
	SPARSDR_ENABLE_COMPRESSION,
	SPARSDR_RUN_FFT,
	SPARSDR_SEND_FFT_SAMPLES,
	SPARSDR_SEND_AVERAGE_SAMPLES,
	SPARSDR_FFT_SIZE,
	SPARSDR_FFT_SCALING,
	SPARSDR_BIN_MASK,
	SPARSDR_THRESHOLD_BIN_NUMBER,
	SPARSDR_THRESHOLD_VALUE,
	SPARSDR_AVERAGE_WEIGHT,
	SPARSDR_AVERAGE_INTERVAL,
	SPARSDR_WINDOW_VAL,
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
				 struct device_attribute *attr, char *buf)
{
	struct iio_dev *indio_dev = dev_to_iio_dev(dev);
	struct iio_dev_attr *this_attr = to_iio_dev_attr(attr);
	struct sparsdr_private_data *data = iio_priv(indio_dev);

	int ret = 0;

	mutex_lock(&indio_dev->mlock);
	switch ((u32)this_attr->address) {
	case SPARSDR_ENABLE_COMPRESSION:
		ret = sprintf(buf, "%u\n",
			      (unsigned int)data->enable_compression);
		break;
	case SPARSDR_RUN_FFT:
		ret = sprintf(buf, "%u\n", (unsigned int)data->run_fft);
		break;
	case SPARSDR_SEND_FFT_SAMPLES:
		ret = sprintf(buf, "%u\n",
			      (unsigned int)data->send_fft_samples);
		break;
	case SPARSDR_SEND_AVERAGE_SAMPLES:
		ret = sprintf(buf, "%u\n",
			      (unsigned int)data->send_average_samples);
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
	case SPARSDR_THRESHOLD_BIN_NUMBER:
		// This doesn't really have a readable value
		ret = sprintf(buf, "0\n");
		break;
	case SPARSDR_THRESHOLD_VALUE:
		// This doesn't really have a readable value
		ret = sprintf(buf, "0\n");
		break;
	case SPARSDR_AVERAGE_WEIGHT:
		ret = sprintf(buf, "%u\n", (unsigned int)data->average_weight);
		break;
	case SPARSDR_AVERAGE_INTERVAL:
		ret = sprintf(buf, "%u\n", data->average_interval);
		break;
	case SPARSDR_WINDOW_VAL:
		// This doesn't really have a readable value
		ret = sprintf(buf, "0\n");
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
				  const char *buf, size_t len)
{
	struct iio_dev *indio_dev = dev_to_iio_dev(dev);
	struct iio_dev_attr *this_attr = to_iio_dev_attr(attr);
	struct sparsdr_private_data *data = iio_priv(indio_dev);

	int ret = 0;
	u32 mask_threshold_value;

	mutex_lock(&indio_dev->mlock);
	switch ((u32)this_attr->address) {
	case SPARSDR_ENABLE_COMPRESSION:
		ret = strtobool(buf, &data->enable_compression);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_ENABLE_COMPRESSION,
				       (u32)data->enable_compression);
		break;
	case SPARSDR_RUN_FFT:
		ret = strtobool(buf, &data->run_fft);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_RUN_FFT, (u32)data->run_fft);
		break;
	case SPARSDR_SEND_FFT_SAMPLES:
		ret = strtobool(buf, &data->send_fft_samples);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_FFT_SEND,
				       (u32)data->send_fft_samples);
		break;
	case SPARSDR_SEND_AVERAGE_SAMPLES:
		ret = strtobool(buf, &data->send_average_samples);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_AVERAGE_SEND,
				       (u32)data->send_average_samples);
		break;
	case SPARSDR_FFT_SIZE:
		ret = kstrtou32(buf, 10, &data->fft_size);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_FFT_SIZE, data->fft_size);
		break;
	case SPARSDR_FFT_SCALING:
		ret = kstrtou32(buf, 10, &data->fft_scaling);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_FFT_SCALING, data->fft_scaling);
		break;
	case SPARSDR_BIN_MASK:
		ret = kstrtou32(buf, 10, &mask_threshold_value);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_BIN_MASK, mask_threshold_value);
		break;
	case SPARSDR_THRESHOLD_BIN_NUMBER:
		ret = kstrtou32(buf, 10, &mask_threshold_value);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_THRESHOLD_BIN_NUMBER,
				       mask_threshold_value);
		break;
	case SPARSDR_THRESHOLD_VALUE:
		ret = kstrtou32(buf, 10, &mask_threshold_value);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_THRESHOLD_VALUE,
				       mask_threshold_value);
		break;
	case SPARSDR_AVERAGE_WEIGHT:
		ret = kstrtou8(buf, 10, &data->average_weight);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_AVERAGE_WEIGHT,
				       (u32)data->average_weight);
		break;
	case SPARSDR_AVERAGE_INTERVAL:
		ret = kstrtou32(buf, 10, &data->average_interval);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_AVERAGE_INTERVAL,
				       data->average_interval);
		break;
	case SPARSDR_WINDOW_VAL:
		ret = kstrtou32(buf, 10, &mask_threshold_value);
		if (ret < 0) {
			break;
		}
		sparsdr_write_register(REGISTER_WINDOW_VAL,
				       mask_threshold_value);
		break;
	default:
		ret = -EINVAL;
		break;
	}
	mutex_unlock(&indio_dev->mlock);

	return ret ? ret : (ssize_t)len;
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
static IIO_DEVICE_ATTR(threshold_bin_number, S_IRUGO | S_IWUSR,
		       sparsdr_attr_read, sparsdr_attr_write,
		       SPARSDR_THRESHOLD_BIN_NUMBER);
static IIO_DEVICE_ATTR(threshold_value, S_IRUGO | S_IWUSR, sparsdr_attr_read,
		       sparsdr_attr_write, SPARSDR_THRESHOLD_VALUE);
static IIO_DEVICE_ATTR(average_weight, S_IRUGO | S_IWUSR, sparsdr_attr_read,
		       sparsdr_attr_write, SPARSDR_AVERAGE_WEIGHT);
static IIO_DEVICE_ATTR(average_interval, S_IRUGO | S_IWUSR, sparsdr_attr_read,
		       sparsdr_attr_write, SPARSDR_AVERAGE_INTERVAL);
static IIO_DEVICE_ATTR(window_value, S_IRUGO | S_IWUSR, sparsdr_attr_read,
		       sparsdr_attr_write, SPARSDR_WINDOW_VAL);

static struct attribute *sparsdr_attributes[] = {
	&iio_dev_attr_enable_compression.dev_attr.attr,
	&iio_dev_attr_run_fft.dev_attr.attr,
	&iio_dev_attr_send_fft_samples.dev_attr.attr,
	&iio_dev_attr_send_average_samples.dev_attr.attr,
	&iio_dev_attr_fft_size.dev_attr.attr,
	&iio_dev_attr_fft_scaling.dev_attr.attr,
	&iio_dev_attr_bin_mask.dev_attr.attr,
	&iio_dev_attr_threshold_bin_number.dev_attr.attr,
	&iio_dev_attr_threshold_value.dev_attr.attr,
	&iio_dev_attr_average_weight.dev_attr.attr,
	&iio_dev_attr_average_interval.dev_attr.attr,
	&iio_dev_attr_window_value.dev_attr.attr,
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

static int sparsdr_init(void)
{
	int register_status;
	struct sparsdr_private_data *data;
	// Allocate memory for the device
	g_sparsdr_iio_dev =
		iio_device_alloc(sizeof(struct sparsdr_private_data));
	if (g_sparsdr_iio_dev == NULL) {
		return -ENOMEM;
	}
	// Fill in attributes
	g_sparsdr_iio_dev->name = "sparsdr";
	g_sparsdr_iio_dev->info = &sparsdr_iio_info;

	// Initialize default values in private data section
	data = iio_priv(g_sparsdr_iio_dev);
	data->enable_compression = true;
	data->run_fft = true;
	data->send_fft_samples = true;
	data->send_average_samples = true;
	// This is really the base-2 logarithm of the FFT size
	data->fft_size = 10;
	data->fft_scaling = 0x6ab;
	data->average_weight = 224;
	data->average_interval = 16;

	register_status = iio_device_register(g_sparsdr_iio_dev);

	return register_status;
}

static void sparsdr_exit(void)
{
	iio_device_unregister(g_sparsdr_iio_dev);
	iio_device_free(g_sparsdr_iio_dev);
	g_sparsdr_iio_dev = NULL;
}

module_init(sparsdr_init);
module_exit(sparsdr_exit);
