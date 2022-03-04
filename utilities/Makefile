#IIO Parameters
IP_ADDR=192.168.2.1
BUFFSIZE=32768
TIMEOUT=0
SAMPLES=300000
FILENAME=samples.dat
AVGFOLDERNAME=/tmp/plutoSparSDRFiles

#Radio Parameters
LOFREQ=2412000000
#Set RXGain = -1 for fast_attack AGC
RXGAIN=60
RFBW=56000000
GAINCONTROLMODE=manual

#SparSDR Specific Registers
SHIFTAMOUNT=4
AVERAGEWEIGHT=30 # corresponds to ~16 point moving average
AVERAGEINTERVAL=9 # send averages every 2**12 windows
ENABLECOMPRESSION=1 # enable SparSDR compression
V2FLAG=True # Version 2 Flag
PLOTFLAG=True # Plot flag

#Signal Specific Configurations
PAPRDB=0
BWMHZ=1


tune:
	iio_attr -u ip:${IP_ADDR} -o -c ad9361-phy voltage0 sampling_frequency 61440000
	iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 rf_bandwidth ${RFBW} 
	iio_attr -u ip:${IP_ADDR} -o -c ad9361-phy altvoltage0 frequency ${LOFREQ}
	make setgain

setgain:
	if [ ${RXGAIN} -eq -1 ]; \
	then iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 gain_control_mode fast_attack; \
	else iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 gain_control_mode manual; \
	iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 hardwaregain ${RXGAIN}; \
	fi; 

test:
	iio_attr -u ip:${IP_ADDR} -o -c ad9361-phy voltage0 sampling_frequency
	iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 rf_bandwidth 
	iio_attr -u ip:${IP_ADDR} -o -c ad9361-phy altvoltage0 frequency 
	iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 gain_control_mode
	iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 hardwaregain
	iio_readdev -n ${IP_ADDR} -b ${BUFFSIZE} -T ${TIMEOUT} cf-ad9361-lpc | pv  > /dev/null

configure:
	python3 pluto_sparsdr_main.py

sample:
	iio_readdev -u ip:${IP_ADDR} -b ${BUFFSIZE} -T ${TIMEOUT} -s ${SAMPLES} cf-ad9361-lpc > ${FILENAME}

samplemon:
	iio_readdev -u ip:${IP_ADDR} -b ${BUFFSIZE} -T ${TIMEOUT} -s ${SAMPLES} cf-ad9361-lpc | pv > ${FILENAME}

view:
	hexdump samples.dat | less 

changemode:
	iio_attr -u ip:${IP_ADDR} -d sparsdr enable_compression ${ENABLECOMPRESSION}

calgainwithant:
	iio_attr -u ip:${IP_ADDR} -d sparsdr enable_compression 0
	make setgain 
	make sample SAMPLES=6000000 BUFFSIZE=1000000 FILENAME=${AVGFOLDERNAME}/clipCheck.iq; \
	NEWVAR=`python3 autoThreshComputePluto.py --clipCheck True --rxgain ${RXGAIN}`; \
	if [ ${RXGAIN} -ne $${NEWVAR} ]; \
	then make calgainwithant RXGAIN=$${NEWVAR}; \
	fi
	iio_attr -u ip:${IP_ADDR} -d sparsdr enable_compression 1
	CORRECTGAIN=`iio_attr -u ip:${IP_ADDR} -i -c ad9361-phy voltage0 hardwaregain`; \
	echo Use gain $${CORRECTGAIN}

getavgloop:
	if [ ! -d ${AVGFOLDERNAME} ]; then mkdir ${AVGFOLDERNAME}; fi;
	iio_attr -u ip:${IP_ADDR} -d sparsdr enable_compression 1
	make setgain
	for c in 7 6 5 4 3 2 1 0; do \
		iio_attr -u ip:${IP_ADDR} -d sparsdr average_interval ${AVERAGEINTERVAL}; \
		iio_attr -u ip:${IP_ADDR} -d sparsdr average_weight ${AVERAGEWEIGHT}; \
		iio_attr -u ip:${IP_ADDR} -d sparsdr send_fft_samples 0; \
		iio_attr -u ip:${IP_ADDR} -d sparsdr shift_amount $${c}; \
		iio_attr -u ip:${IP_ADDR} -d sparsdr send_average_samples 1; \
		iio_readdev -u ip:${IP_ADDR} -b ${BUFFSIZE} -T ${TIMEOUT} -s 204800 cf-ad9361-lpc | pv > ${AVGFOLDERNAME}/avgSamples.dat_$${c}_${RXGAIN}; \
	done

	python3 autoThreshComputePluto.py --bwmhz ${BWMHZ} --papr ${PAPRDB} --rxgain ${RXGAIN} --plot ${PLOTFLAG} --v2 ${V2FLAG} --avgFolder ${AVGFOLDERNAME}
