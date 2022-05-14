# Calibrating the SparSDR on Pluto-SDR


The noise levels at the output of SparSDR should be measured to ascertain the correct thresholds to set.

## Steps to calibrate SparSDR

In order to properly calibrate the radio, the following steps have to performed in order.

1. Step 1 (With antenna): `make tune LOFREQ=<LOFREQ> RXGAIN=<GaniofChoice>`    
2. Now the Pluto-SDR is set to the required center frequency with a gain of choice
3. Step 2 (With antenna): `make calgainwithant RXGAIN=<GainofChoice>`
4. The Pluto-SDR searches over possible gains to set that avoid receiver saturation in the given environment. It then suggests a gain to use. Make sure the Pluto-SDR can hear the signals you want to listen to during this step.
5. Step 3 (Without antenna): `make getavgloop RXGAIN=<SuggestedGain>`
6. Step 3 calibrates the median noise floor for the given gain. Then, it creates a file: `thresholdConfig.txt`. The text file contains the suggested shift value and the median noise floor levels for the given gain and suggested shift value.
7. Step 4: Use the python script `SparSDR_offset_threshold_create.py` to create a new file where the threshold is set 10 dB above the median noise floor. This script can be appropriately modified to use the median noise floor to choose the right threshold.
8. Step 5: Use your threshold Configuration text file in the GNURadio block. Set the "Threshold source" to "Threshold File" and provide the path to the text file. 

Now SparSDR can be used with the antenna connected.