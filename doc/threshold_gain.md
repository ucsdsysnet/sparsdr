# Setting the threshold and gain

## Understanding gain

The USRP can increase or decrease the amplitude of signals before they reach
the analog-to-digital converter. Gain values are in decibels (dB). The maximum
effective gain for the USRP N210 is 31.5 dB. The USRP will accept higher
gain values, but it will provide the higher gain by multiplying the digital
samples. This does not help with signal quality, so it is usually best avoided.

## Understanding threshold

The compression process on the FPGA calculates the overall amplitude in each
frequency bin as the sum of the real amplitude squared and the imaginary
amplitude squared. The maximum possible overall amplitude is approximately
1 billion. The FPGA sends to the host computer all frequency bins with
overall amplitudes greater than the threshold.

## Choosing a threshold and gain

Start with a relatively high gain. The higher the gain, the larger the signal
amplitudes in the compression process will be. This will provide higher
resolution when comparing amplitudes to the threshold.

Start with a threshold around 1000. Increase the threshold if you encounter
overflow, and decrease the threshold if you are not receiving the signals
you are interested in.

### Overflow

When the USRP tries to send more compressed data than the network or host
computer can handle, it enters an overflow state and stops sending data.

When this happens, up to a second of signals will be missing from the output.
If overflow is frequent, you will need to increase the threshold.

### Missing signals

If the threshold is too high, the USRP may not send any data to the host
computer. It will send only the average samples, which will result in a
very small network traffic rate.

## Example values

For Bluetooth signals sent from a device less than one meter from the receiver,
a gain of 30 and a threshold of 10000 is a good place to start.
