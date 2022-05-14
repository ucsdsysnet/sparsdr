#!/usr/bin/env python3

from sys import stdin
from struct import unpack
import warnings
import numpy as np

def parsePlutoV2(filename,fft_size_log2=10):
  '''
  Function that parses and returns a compressed capture
  from a Pluto in the V2 format.
  :input: filename :string: Full path to compressed file
  :input: fft_size_log2 :int: log2(NFFT), has to be 1024 or less

  :output: magIdxList :list:int: Bin Index of magnitude average value 
  :output: fixedAvgTimeList :list:int: Time corresponding to average window
  :output: avgMagnitudeList :list:int: Magnitude average value per bin
  :output: fftNoList :list:int: The index of the FFT window
  :output: fftIndexList :list:int: The index of the bin within the FFT window
  :output: fixedFftTimeList :list:int: Time corresponding to the FFT window
  :output: realList :list:int: Real part of bin value
  :output: imagList :list:int: Imaginary part of bin value
  '''

  max_fft_size_log2 = 10
  FFT_size          = 2**fft_size_log2

  fft_time_offset = int(0)
  avg_time_offset = int(0)
  last_fft_time   = int(0)
  last_avg_time   = int(0)

  first_zero = 0
  after_zero = False
  in_FFT     = True
  in_avg     = False
  after_hdr  = False
  FFT_index  = 0

  # Clock is 61.44MHz, and we cut (fft_size_log-1) bits to show start of window
  ts = 16.2760417 * (1 << (fft_size_log2-1))

  v1_time_bits  = 32-1-max_fft_size_log2
  v1_time_mask  = (2**(v1_time_bits))-1

  magIdxList = []
  fixedAvgTimeList = []
  avgMagnitudeList = []

  fftNoList = []
  fftIndexList = []
  fixedFftTimeList = []
  realList = []
  imagList = []

  fhandle = open(filename, 'rb')
  while True:
    b = fhandle.read(4)
    if (len(b) < 4):
      break

    (value, )    = unpack("I", b)
    (imag, real) = unpack("hh", b)

    is_hdr = (value >> 31) & 0x1
    is_avg = (value >> 30) & 0x1
    time   = (value & 0x3FFFFFFF);
    index  = (value & 0x3FF)
    fft_no = time & 0x1 # simply odd or even time for start of window
    
    # print ("if hdr:", is_hdr, is_avg, time, "if data:", value)

    if (first_zero!=2):
      if (first_zero==0):
        if (value==0):
          first_zero = 1
        print ("Trying to find a proper header")
        continue
      else: # potential first zero
        if (value==0):
          print ("Trying to find a proper header")
          continue
        elif not is_hdr:
          print ("Trying to find a proper header")
          first_zero = 0 # reset the search
          continue
        else:
          first_zero = 2
          after_zero = True
          fft_time_offset = -time
          avg_time_offset = -time

    if (in_FFT and (value==0)) or (in_avg and (FFT_index==FFT_size) and (value==0)):
      after_zero = True
      # print ("(End Frame)")
      continue

    if (after_zero):
      if (value==0):
        first_zero=1
        print ("Error detecting window")
      elif is_hdr:
        if is_avg:
          if (time < last_avg_time):
            avg_time_offset += (1<<30)
          last_avg_time = time
          # Average sample times always have fft_size_log bits tail zero
          fixed_avg_time = ((time & 0x3FFFFFFE) + avg_time_offset) * ts
          # print ("Average header at time", fixed_avg_time,"(ns)")
          FFT_index = 0
          in_avg = True
          in_FFT = False

        else:
          if (time < last_fft_time):
            fft_time_offset += (1<<30)
          last_fft_time = time
          fixed_fft_time = (time + fft_time_offset) * ts
          # print ("FFT header at time", fixed_fft_time,"(ns)")
          in_avg = False
          in_FFT = True
          after_hdr = True
      else:
        # print ("(FFT index)")
        FFT_index = index
      after_zero = False
    else:
      if (in_avg):
        # print ("Average, index", FFT_index, ":", value)
        magIdxList.append(FFT_index)
        avgMagnitudeList.append(index)
        fixedAvgTimeList.append(fixed_avg_time)
        FFT_index += 1
      else:
        if (after_hdr):
          FFT_index = index
          # print ("(FFT index)")
          after_hdr = False
        else:
          # print ("FFT, index", FFT_index, ":", real, ",", imag, "( power =", (real*real)+(imag*imag), ")")
          fftNoList.append(time)
          fftIndexList.append(FFT_index)
          fixedFftTimeList.append(fixed_fft_time)
          realList.append(real)
          imagList.append(imag)

          FFT_index += 1
  fhandle.close()
  return magIdxList, fixedAvgTimeList, avgMagnitudeList, fftNoList, fftIndexList, fixedFftTimeList, realList, imagList

def parsePlutoV1(filename,fft_size_log2=10):
  '''
  Function that parses and returns a compressed capture
  from a Pluto in the V1 format.
  :input: filename :string: Full path to compressed file
  :input: fft_size_log2 :int: log2(NFFT), has to be 1024 or less

  :output: magIdxList :list:int: Bin Index of magnitude average value 
  :output: fixedAvgTimeList :list:int: Time corresponding to average window
  :output: avgMagnitudeList :list:int: Magnitude average value per bin
  :output: fftNoList :list:int: The index of the FFT window
  :output: fftIndexList :list:int: The index of the bin within the FFT window
  :output: fixedFftTimeList :list:int: Time corresponding to the FFT window
  :output: realList :list:int: Real part of bin value
  :output: imagList :list:int: Imaginary part of bin value
  '''

  max_fft_size_log2 = 10

  n               = int(0)
  fft_time_offset = int(0)
  avg_time_offset = int(0)
  last_fft_time   = int(0)
  last_avg_time   = int(0)

  index_mask = (2**max_fft_size_log2)-1
  time_bits  = 32-1-max_fft_size_log2
  time_mask  = (2**(time_bits))-1

  # Clock is 61.44MHz, and we cut (fft_size_log-1) bits to show start of window
  ts = 16.2760417 * (1 << (fft_size_log2-1))

  # print ("#%-9s | %9s | %9s | %9s | %9s | %9s | %9s" % ("Sample", "Type", "FFT_No", "Index", "Time(ns)", "Real", "Imag"))

  fhandle = open(filename, 'rb')

  magIdxList = []
  fixedAvgTimeList = []
  avgMagnitudeList = []

  fftNoList = []
  fftIndexList = []
  fixedFftTimeList = []
  realList = []
  imagList = []

  while True:
    b = fhandle.read(8)
    if (len(b) < 8):
      break

    (imag, real, hdr)  = unpack("hhI", b)
    (avg_magnitude, _) = unpack("II", b)

    is_avg = (hdr >> 31) & 0x1
    index  = (hdr >> time_bits) & index_mask
    time   = (hdr & time_mask)
    fft_no = time & 0x1 # simply odd or even time for start of window

    if (n==0):
      fft_time_offset = -time
      avg_time_offset = -time
    n = n + 1

    if (is_avg):
      if (time < last_avg_time):
        avg_time_offset += (1<<time_bits)
        # print ("Average window time overflow: %d -> %d" % (last_avg_time, time))
      last_avg_time = time
      # Average sample times always have fft_size_log bits tail zero
      fixed_avg_time = ((time & (time_mask-1)) + avg_time_offset) * ts

      magIdxList.append(index)
      fixedAvgTimeList.append(fixed_avg_time)
      avgMagnitudeList.append(avg_magnitude)

    else:
      if (time < last_fft_time):
        fft_time_offset += (1<<time_bits)
        # print ("FFT window time overflow: %d -> %d" % (last_fft_time, time))
      last_fft_time = time
      fixed_fft_time = (time + fft_time_offset) * ts
      print ("% -10d    FFT sample % 10d  % 10d  % 10d  % 10d  % 10d (power=% 10d)" % (n, fft_no, index, fixed_fft_time, real, imag, (real*real)+(imag*imag)))

      fftNoList.append(fft_no)
      fftIndexList.append(index)
      fixedFftTimeList.append(fixed_fft_time)
      realList.append(real)
      imagList.append(imag)

  fhandle.close()
  return magIdxList, fixedAvgTimeList, avgMagnitudeList, fftNoList, fftIndexList, fixedFftTimeList, realList, imagList

def read_cshort_binary(filename):
  '''
  Read iio_readdev raw binary capture from pluto with 
  SparSDR disabled.
  :input: filname :string: Full path to file
  '''
  fhandle = open(filename, 'rb')

  realList = []
  imagList = []

  while True:
    b = fhandle.read(4)
    if (len(b) < 4):
      break
    (real, imag)  = unpack("hh", b)
    realList.append(real)
    imagList.append(imag)

  fhandle.close()

  complexArray = np.asarray(realList,dtype=float) + 1j * np.asarray(imagList,dtype=float)

  return complexArray

def conservativeShiftPluto(nfft, estPAPRdB = 0, estBWMHz=1):
  '''
  Computes the conservative shift value in the Pluto using the input
  PAPR and Bandwidth of the highest power signal in band. This 
  computation assumes that the peak power of this signal is exactly
  fit into the dynamic range of the ADC (12 bits). The computation 
  also assumes that the sample rate is 61.44 MHz, and that the 
  most conservative bit shift is 7.
  :input: nfft :int: Number of bins in FFT
  :input: estPAPRdB :float: Peak to Average Power Ratio of the 
  strongest signal in band (dB).
  :input: estBWMHz :float: Bandwidth of the strongest signal in band
  in MHz.
  :output: conservativeShift :float: conservative shift based on 
  given inputs.
  '''
  # Get max conservative shift
  sampRateMHz = 61.44;
  estBinsinBW = np.ceil(nfft*estBWMHz/sampRateMHz);
  conservativeShift = 7-np.log2(10**((estPAPRdB+10*np.log10(estBinsinBW))/20)); # db2mag
  # conservativeShift = int(conservativeShift)

  return conservativeShift

def computeShiftThresholdsPluto(nfft, rxGain, SparSDRVersion = 1, plutoSDRAvgFolder='/tmp/plutoSparSDRAvgValueFiles', filenameRxTemplate = 'avgSamples.dat'):
  '''
  Compute suggested shift value and noise floor calibration 
  from average magnitude SparSDR captures performed without
  antenna connected.
  :input: nfft :int: Number of points in the FFT
  :input: rxGain :float: The Rx Gain of the Pluto to use
  :input: SparSDRVersion :int: Version of SparSDR used
  :input: plutoSDRAvgFolder :string: Folder containing the
  average file captures for each shift value
  :input: filenameRxTemplate :string: Template filename of
  each of the average file captures

  :output: shiftValue :int: Suggested shift value that 
  brings the analog noise into the quantization range
  :output: binIdx :nparray:int: Index of bins
  :output: threshLinear :nparray:int: Array containting 
  linear thresholds corresponding to each entry in binIdx
  :output: threshLinearOutliers :nparray:int: Array 
  containing linear threshold values corresponding to out-
  lier bins. If a bin is not an outlier, then the value of
  this array at that location will be nan
  '''
  # shiftValue, binIdx, threshLinear, threshLinearOutliers, smooth2SidedMediandB

  fftSizeLog2 = int(np.log2(nfft));
  binVec = range(nfft)

  numShifts = 8;

  shiftValueVec = np.arange(7,-1,-1,dtype=int);
  rxGainVec = rxGain*np.ones((8,),dtype=int);

  for idx in range(numShifts):
    shiftValue = shiftValueVec[idx]
    rxGain = rxGainVec[idx]
    fileName = plutoSDRAvgFolder + '/' + filenameRxTemplate + '_' + str(shiftValue) + '_' + str(rxGain)

    if(SparSDRVersion==1):
      magIdxList, fixedAvgTimeList, avgMagnitudeList = parsePlutoV1(fileName, fftSizeLog2)[0:3]
    else:
      magIdxList, fixedAvgTimeList, avgMagnitudeList = parsePlutoV2(fileName, fftSizeLog2)[0:3]

    magIdxList = np.asarray(magIdxList)
    fixedAvgTimeList = np.asarray(fixedAvgTimeList)
    avgMagnitudeList = np.asarray(avgMagnitudeList)

    lengthAvgMagList = avgMagnitudeList.shape[0]
    lengthAvgMagListRound = int(np.floor(lengthAvgMagList/nfft)*nfft)
    avgMagnitudeList = avgMagnitudeList[:lengthAvgMagListRound]

    avgMagnitudeMat = avgMagnitudeList.reshape((-1,nfft))
    avgMagnitudeMat = avgMagnitudeMat - 1
    medianVals = np.median(avgMagnitudeMat,axis=0)
    medianVals = np.fft.fftshift(medianVals)
    medianValsdB = 10*np.log10(medianVals)

    convKernel = np.ones((8,))/8;
    smooth2SidedMedian = np.convolve(medianVals,convKernel,mode='same')
    smooth2SidedMediandB = 10*np.log10(smooth2SidedMedian);

    p = np.polyfit(binVec, smooth2SidedMediandB, 2);
    y1 = np.polyval(p,binVec);

    if(np.sum((smooth2SidedMediandB)<0)==0):
      break
    
  estimError = smooth2SidedMediandB - y1;
  y2 = y1.copy();
  y2[estimError<4] = np.nan
  binIdx = np.fft.fftshift(binVec)
  threshLinear = np.ceil(10**(y1/10))
  threshLinearOutliers = np.ceil(10**((y2+estimError)/10))

  return shiftValue, binIdx, threshLinear, threshLinearOutliers, smooth2SidedMediandB

def saveConfigtoFilePluto(rxGain,estPAPRdB,estBWMHz,conservativeShiftValue,shiftValue,binIdx,threshLinear,threshLinearOutliers,filename='thresholdConfig.txt'):
  '''
  Saves threshold/noise configuration parameters to a file
  :input: rxGain :float: RxGain in dB
  :input: estPAPRdB :float: PAPR of strongest signal in dB
  :input: estBWMHz :float: Bandwidth of the strongest sig-
  nal in Hz.
  :input: conservativeShiftValue :float: Conservative shift
  value.
  :input: shiftValue :int: Suggested shift value that 
  brings analog noise into quantization range.
  :input: binIdx :nparray:int: Bin indices as an array
  :input: threshLinear :nparray:int: Linear thresholds for 
  each entry in binIdx
  :input: threshLinearOutliers :nparray:int: Linear thresh-
  olds for the outlier bins. Non-outlier values are nan
  :input: filename :string: Filename to store the calibrat-
  ion configuration.
  '''
  fhandle = open(filename, 'w');

  fhandle.write('RxGaindB '+str(rxGain)+'\n')
  fhandle.write('EstPAPRdB '+str(estPAPRdB)+'\n')
  fhandle.write('estBWMHz '+str(estBWMHz)+'\n')
  fhandle.write('ConservativeShift '+str(conservativeShiftValue)+'\n')
  fhandle.write('SuggestedShift '+str(shiftValue)+'\n')

  for binIter, threshIter, threshOutlierIter in zip(binIdx, threshLinear, threshLinearOutliers):
    # print(binIter,threshIter,threshOutlierIter)
    if not np.isnan(threshOutlierIter):
      # print(binIter,threshIter,threshOutlierIter)
      fhandle.write(str(binIter)+' '+str(threshOutlierIter)+'\n')
    else:
      # print(binIter,threshIter,threshOutlierIter)
      fhandle.write(str(binIter)+' '+str(threshIter)+'\n')

  # pdb.set_trace();
  fhandle.close()

def getThresholdBinShiftFromFile(filename='./thresholdConfig.txt', thresholdOffsetdB = 6, conserveShift = False):
  '''
  Reads the noise floor calibration from a file and uses other
  inputs to determine the thresholds to use.
  :input: filename :string: Noise floor calibration/threshold 
  calibration file
  :input: thresholdOffsetdB :float: Threshold offset from the 
  calibrated noise floor in dB. If this number is set higher, 
  SparSDR will not allow weak powered signals.
  :input: conserveShift :bool: If set to True, the function 
  will use the most conservative shift. If set to false, the 
  function will use the suggested shift that brings the analog 
  noise into the quantization range. Will throw a warning if 
  we expect to lose dynamic range, or risk numerical overflows

  :output: RxGain :float: RxGain in dB read from the input file
  :output: binShift :int: shift value chosen according to inputs
  :output: binIdx :nparray:int: Bin indices corresponding to 
  thresholds
  :output: threshVec :nparray:int: Threshold values correspond-
  ing to the bin indices.
  '''

  fhandle = open(filename,'r');

  line1 = fhandle.readline() # rxgain
  line2 = fhandle.readline() # EstPAPRdB
  line3 = fhandle.readline() # estBWMHz
  line4 = fhandle.readline() # ConservativeShift
  line5 = fhandle.readline() # SuggestedShift

  RxGain = int(line1.split()[1])
  ConservativeShift = float(line4.split()[1])
  SuggestedShift = int(line5.split()[1])
  # RxGain = line2.split()[1]

  binIdx = []
  threshVec = []
  # Get the thresholds and bins
  for line in fhandle:
    binIdx.append(int(line.split()[0]))
    threshVec.append(float(line.split()[1]))    

  fhandle.close()

  binIdx = np.asarray(binIdx)
  threshVec = np.asarray(threshVec,dtype=float)

  if(conserveShift):
    binShift = np.ceil(ConservativeShift)
  else:
    binShift = np.ceil(SuggestedShift)

  binShift = binShift.astype(int)

  if(binShift < ConservativeShift):
    warnings.warn("Using a shift value that is lower than the conservative shift estimate. Numerical overflows may occur. Conservative Shift: " + str(ConservativeShift) + ', Used Shift: ' + str(binShift))

  divDifference = binShift - SuggestedShift

  if divDifference > 0:
    dynRangeLoss = 20*np.log10(2**divDifference)
    warnings.warn("Using a shift value higher than suggested value. Estimated dynamic range loss: "+str(dynRangeLoss)+" dB. Suggested Shift: " + str(SuggestedShift) + ', Used Shift: ' + str(binShift))

  threshVec = np.round(threshVec*(10**(thresholdOffsetdB/10))/(4**divDifference))
  threshVec = threshVec.astype(int)

  return RxGain, binShift, binIdx, threshVec