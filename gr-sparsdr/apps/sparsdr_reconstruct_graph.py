#!/usr/bin/env python

"""
Demonstrates the SparSDR reconstruct block
"""

from gnuradio import gr, blocks
import sparsdr

def main():
    top_block = gr.top_block()

    # Read compressed samples from a file
    compressed_file = blocks.file_source(gr.sizeof_int, '/home/samcrow/Documents/CurrentClasses/Research/Compression/2018-12-05/car-remote-2.iqz')

    # Reconstruct
    # One band, all 2048 bins centered
    bands = sparsdr.band_spec_vector()
    bands.push_back(sparsdr.band_spec(0, 2048))
    reconstruct = sparsdr.reconstruct(bands=bands, reconstruct_path='/home/samcrow/Documents/CurrentClasses/Research/Compression/rfsniffer/src/decompress/target/release/sparsdr_reconstruct')

    top_block.connect(compressed_file, reconstruct)

    # Ignore reconstructed samples
    reconstructed_sink = blocks.file_sink(gr.sizeof_gr_complex, 'reconstructed.iq')
    top_block.connect(reconstruct, reconstructed_sink)

    print('Running')
    top_block.run()

if __name__ == "__main__":
    main()
