import math

def disable_compression(self):
  self._u.set_user_register(19, 0, 0)

def enable_compression(self):
  self._u.set_user_register(19, 1, 0)

def stop_FFT(self):
  self._u.set_user_register(17, 0, 0)

def start_FFT(self):
  self._u.set_user_register(17, 1, 0)

def stop_FFT_send(self):
  self._u.set_user_register(15, 0, 0)

def start_FFT_send(self):
  self._u.set_user_register(15, 1, 0)

def stop_avg_send(self):
  self._u.set_user_register(16, 0, 0)

def start_avg_send(self):
  self._u.set_user_register(16, 1, 0)

def stop_all(self):
  self._u.set_user_register(17, 0, 0)
  self._u.set_user_register(16, 0, 0)
  self._u.set_user_register(15, 0, 0)

def start_all(self):
  self._u.set_user_register(15, 1, 0)
  self._u.set_user_register(16, 1, 0)
  self._u.set_user_register(17, 1, 0)

# Set FFT size.
# There needs to be a stop_FFT before and a start_FFT after this.
def set_FFT_size(self, size):
  self._u.set_user_register(20, size, 0)

# FFT schedular for each step, first 12 bits are used. Default value is 0x6AB
# There needs to be a stop_FFT before and a start_FFT after this.
def set_FFT_scaling(self, sched):
  self._u.set_user_register(10, sched, 0)

# the value is divided by 2048, since in HW is it shifted left by 11
def set_threshold(self, index, value):
  self._u.set_user_register(11, ((index<<21)+(value>>11)), 0)

def set_window_val(self, index, value):
  self._u.set_user_register(18, ((index<<16)+value), 0)

def reset_mask(self, index):
  self._u.set_user_register(12, (index<<1)+0, 0)

def set_mask(self, index):
  self._u.set_user_register(12, (index<<1)+1, 0)

# set alpha weight in alpha*avg+(1-alpha)*new_sample. It should be
# be less than 1, and it would be digitized to 8 bits (256 possible values)
def set_avg_weight (self, weight):
  self._u.set_user_register(13, int(weight*256)&0x000000FF, 0)

# set frequency of sending average values in term of FFT windows (from 0 t0 2^31)
def set_avg_packet_frequency (self, freq): #0xE0
  self._u.set_user_register(14, int(math.ceil(math.log(freq,2))), 0)
