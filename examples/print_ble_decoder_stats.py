import time
import zmq
import pdb
import struct
import pmt
import re
import logging
logging.basicConfig(level=logging.DEBUG,format='%(asctime)s %(levelname)s:%(message)s')

context = zmq.Context()
#  Socket to talk to server
print("Connecting to ZMQ Pub Server...")
socket = context.socket(zmq.SUB)
socket.connect("tcp://127.0.0.1:69001")
socket.setsockopt(zmq.SUBSCRIBE,b'')

num_tests = 10
start_time = time.time()
# for request in range(num_tests):
while True:
    #  Get the reply.
    message = socket.recv()
    a = pmt.deserialize_str(message)

    b  = re.split(' |\.|\)',str(a))

    current_rate = int(b[3]);
    average_rate = int(b[8]);

    log_msg = f" Compression: {100*(1-current_rate/61.44e6):.3f}% | SparSDR rate: {current_rate/1e6:.3f} MHz | Actual Sample Rate: {61.44} MHz";
    logging.info(log_msg);

end_time = time.time()
time_elapsed = end_time - start_time