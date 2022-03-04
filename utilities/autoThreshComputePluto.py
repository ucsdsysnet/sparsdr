#!/usr/bin/env python3

import SparSDRUtil
import numpy as np
import argparse

def init_argparse() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        usage="%(prog)s [OPTION]",
        description="Compute thresholds for SparSDR and set them"
    )
    parser.add_argument(
        "-v", "--version", action="version",
        version = f"{parser.prog} version 0.1.0"
    )
    parser.add_argument('--clipCheck', type=bool, default = False,
             help='Flag to trigger function that checks time domain ADC overflow')
    parser.add_argument('--avgFolder', type=str, default = '/tmp/plutoSparSDRFiles/',
             help='Folder where the average calibration files are located')
    parser.add_argument('--v2', type=bool, default = False,
         help='Enable v2 function')
    parser.add_argument('--nfft', type=int, default = 1024,
         help='FFT Size, defaults to 1024')
    parser.add_argument('--rxgain', type=int, default = 30,
        help='RX Gain to use, defaults to 30')
    parser.add_argument('--papr', type=float, default = 0,
         help='Estimated PAPR of strongest input signal in dB, defaults to 0')
    parser.add_argument('--bwmhz', type=float, default = 0.01,
         help='Estimated banwidth of strongest input signal in MHz, defaults to 0.01')
    parser.add_argument('--plot', type=bool, default = False,
     help='Set this flag to true to visualize the threshold computation')
    return parser

def autoComputeThreshold(args):
    '''
    Automatically compute threshold based on given arguments.
    '''

    # Calibration file logistics
    plutoSDRAvgFolder = args.avgFolder
    filenameRxTemplate = 'avgSamples.dat'

    # SparSDR and SDR Frontend parameters
    if args.v2:
        SparSDRVersion = 2
    else:
        SparSDRVersion = 1

    nfft = args.nfft;
    numShifts = 8; # Number of shifts to iterate over
    rxGain = args.rxgain;

    # Signal specific parameters
    estPAPRdB = args.papr
    estBWMHz = args.bwmhz

    conservativeShiftValue = SparSDRUtil.conservativeShiftPluto(nfft, estPAPRdB = estPAPRdB, estBWMHz = estBWMHz)
    shiftValue, binIdx, threshLinear, threshLinearOutliers, smooth2SidedMediandB = SparSDRUtil.computeShiftThresholdsPluto(nfft, rxGain, SparSDRVersion=SparSDRVersion, plutoSDRAvgFolder=plutoSDRAvgFolder, filenameRxTemplate = filenameRxTemplate)

    SparSDRUtil.saveConfigtoFilePluto(rxGain,estPAPRdB,estBWMHz,conservativeShiftValue,shiftValue,binIdx,threshLinear,threshLinearOutliers,filename='thresholdConfig.txt')

    if args.plot:
        import matplotlib.pyplot as plt
        plt.plot(10**(smooth2SidedMediandB/10))
        plt.plot(threshLinearOutliers)
        plt.plot(threshLinear)  
        plt.xlabel('Centered Bin Index')
        plt.ylabel('Power (Linear)')
        plt.legend(('Median Noise Power','Outlier thresholds','PolyFit thresholds'))
        plt.show()

def clipCheck(args):
    filenameRx = args.avgFolder + "/clipCheck.iq";
    rxSamples = SparSDRUtil.read_cshort_binary(filenameRx);

    maxReal = np.max(np.abs(np.real(rxSamples)))
    maxImag = np.max(np.abs(np.imag(rxSamples)))

    clipMax = 2000

    if(maxImag > clipMax or maxReal > clipMax):
        # print('ADC Clipping Detected!')
        # print('Max Real', maxReal, ', Max Imag', maxImag)
        newGaindB = int(args.rxgain/2)
        return newGaindB
    else:
        maxValue = np.max([maxImag, maxReal])
        dBConversion = 20*np.log10(clipMax/maxValue)
        if(dBConversion<2):
            return args.rxgain
        else:
            return int(args.rxgain+dBConversion)

def main():
    parser = init_argparse()
    args = parser.parse_args()

    if args.clipCheck:
        retVal = clipCheck(args)
        print(retVal)
    else:
        autoComputeThreshold(args)


if __name__ == "__main__":
    main()
