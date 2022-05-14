#!/usr/bin/env python3


import SparSDRUtil


filename='./thresholdConfig.txt'
thresholdOffsetdB = 10
conserveShift = False

RxGain, binShift, binIdx, threshVec = SparSDRUtil.getThresholdBinShiftFromFile(filename, thresholdOffsetdB, conserveShift)


filenameNew = './thresholdConfig10dBoffset.txt'

SparSDRUtil.saveConfigtoFilePluto(RxGain,0,1,binShift,binShift,binIdx,threshVec,threshVec,filenameNew)