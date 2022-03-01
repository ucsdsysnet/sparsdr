#!/usr/bin/env ruby

=begin
This file contains various functions used to create and write 32-bit samples. They can be used to create test files
for v2 parser tests.
=end

HEADER_VALID_BIT = 1 << 31
HEADER_AVERAGE_BIT = 1 << 30
TIME_MASK = 0x3fffffff

GROUP_HEADER_HAS_SEQUENCE_BIT = 0x40000000
GROUP_HEADER_SEQUENCE_MASK = 0x3fff
GROUP_HEADER_SEQUENCE_SHIFT = 16
GROUP_HEADER_BIN_MASK = 0xffff

BIN_VALUE_MASK = 0xffff

# Creates and returns a header for an FFT window with the provided time
def fft_header(time)
    HEADER_VALID_BIT | (time & TIME_MASK)
end

# Creates and returns a header for an average window with the provided time
def average_header(time)
    HEADER_VALID_BIT | HEADER_AVERAGE_BIT | (time & TIME_MASK)
end

def group_header(bin, sequence=nil)
    if (bin & GROUP_HEADER_BIN_MASK) != bin
        raise 'Bin number too large'
    end

    sequence_bits = 0
    if sequence
        if (sequence & GROUP_HEADER_SEQUENCE_MASK) != sequence
            raise 'Sequence number too large'
        end

        sequence_bits = GROUP_HEADER_HAS_SEQUENCE_BIT | ((sequence & GROUP_HEADER_SEQUENCE_MASK) << GROUP_HEADER_SEQUENCE_SHIFT)
    end
    sequence_bits | (bin & GROUP_HEADER_BIN_MASK)
end

def bin_value(real, imaginary)
    bytes = [imaginary, real].pack('vv')
    bytes.unpack('V')[0]
end

samples = [
    0x0,
    fft_header(132),
    group_header(2, 13),
    bin_value(32066, -32768),
    bin_value(-9999, 31000),
    0x0,
    group_header(5, 13),
    bin_value(120, 121),
    bin_value(37, -31000),
    0x0,
    average_header(132),
    1, 20, 30, 40, 99, 79, 69, 59,
    0x0,
#     fft_header(133),
#     group_header(0, 14),
#     bin_value(83, 93),
#     bin_value(84, 94),
#     bin_value(85, 95),
#     bin_value(86, 96),
#     bin_value(87, 97),
#     bin_value(88, 98),
#     bin_value(89, 99),
#     bin_value(90, 100),
#     0x0,
    fft_header(150),
    group_header(1, 15),
    bin_value(52, 12),
    0x0,
    fft_header(160),
    group_header(7, 16),
    bin_value(-1, -1),
    0x0,
    fft_header(135),
]

# Output section
# Set to binary mode
STDOUT.binmode()

# Converts a 32-bit sample to little-endian bytes and writes the bytes to standard output
def stdout_write_sample(sample)
    STDOUT.write([sample].pack('V'))
end

samples.each {|sample| stdout_write_sample(sample) }
