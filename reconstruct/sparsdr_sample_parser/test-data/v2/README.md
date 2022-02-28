# SparSDR v2 parser test data

These files can be used to test a SparSDR v2 compressed sample parser.

## Files

Each test case contains a `.json` file with test information and the expected output of the parser.
There is also a corresponding `.iqz` file with the compressed samples to be parsed.

### JSON file format

Each test file must contain a JSON object with these fields:

* `compressed_file`: The path (relative to the JSON file) of the compressed sample file to read and parse
* `fft_size`: The number of bins in the compressed sample file (this can be used to configure the parser)
* `expected_windows`: A list of window objects that represent what the parser should produce

### Window object format

If a window object contains only the key `error` corresponding to an empty object `{}`, the window object
represents an error. This means that the parser should report an error instead of a valid window.

Otherwise, the window object represents an average or FFT window. The window object must have the key `time`
corresponding to the expected window timestamp (an integer number of FFTs).

If the expected window contains FFT samples, it must have the key `bins` corresponding to a list of integer complex
numbers. The number of bins should be the same as the FFT size. Each complex number is represented as a
list of two integers, with the real part followed by the imaginary part. For example, `-30 + 3i` appears in JSON
as `[-30, 3]`.

If the expected window contains averages, it must have the key `averages` corresponding to a list of integer
average values. The number of averages should be the same as the FFT size.
