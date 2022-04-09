#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#define COPY_AVG 0
#define VERBOSE 0
#define AVG_ZERO_CHK 0
#define IDX_HDR_CHK 0
const unsigned int fft_size = 8;

#define HDR_BIT 0x80000000
#define AVG_BIT 0x40000000
#define TS_MASK 0x3FFFFFFF

unsigned int buf_size = 3276800;
unsigned int sample_buf [3276800];

// Only valid states, Error is handled separately
// WIN_HDR: We expect a window header after average window (or initially)
// IDX_HDR: After FFT header we expect an index header
// ANY_HDR: After delimiter in FFT it could be new window or index
// FFT: FFT value or delimiter, AVG: AVG value or delimiter
enum states {WIN_HDR, IDX_HDR, ANY_HDR, FFT, AVG} state;

unsigned short fft_index;
unsigned int   ts;
unsigned short seq_num;

// Returns -1 for ERROR, 1 for beginning of average values, 0 in normal mode
int parse_word (unsigned int word) {
  unsigned short idx;

  switch (state){
    case ANY_HDR:
      if (word & HDR_BIT){
        ts = word & TS_MASK;
        if (word & AVG_BIT){
          printf("Average header at time stamp 0x%08x \n", ts);
          state = AVG;
          fft_index = 0;
          return 1;
        } else {
          printf("FFT header at time stamp 0x%08x \n", ts);
          state = IDX_HDR;
        }
      } else {
        idx     = (unsigned short) (word & 0xFFFF);
        seq_num = (unsigned short) ((word>>16) & 0x3FFF);
        if (VERBOSE) printf("(FFT index header, seq num: %d)\n", seq_num);

        if (IDX_HDR_CHK)
          if (!(word&0x40000000)) {
            printf ("ERROR in FFT index %d, missing idx header bit.\n", idx);
            return -1;
          }

        if ((idx >= fft_size)||(idx <= fft_index)) {
          printf ("ERROR in FFT index %d\n", idx);
          return -1;
        }
        fft_index = idx;
        state = FFT;
      }
      break;

    case WIN_HDR:
      if (word & HDR_BIT){
        ts = word & TS_MASK;
        if (word & AVG_BIT){
          printf("Average header at time stamp 0x%08x \n", ts);
          state = AVG;
          fft_index = 0;
          return 1;
        } else {
          printf("FFT header at time stamp 0x%08x \n", ts);
          state = IDX_HDR;
        }
      } else {
        printf("ERROR: expecting new window after Average window\n");
        return -1;
      }
      break;

    case FFT:
      if (word == 0) {
        state = ANY_HDR;
        if (VERBOSE) printf ("(End Frame)\n");
      } else {
        short imag = (short) (word & 0xFFFF);
        short real = (short) ((word>>16) & 0xFFFF);
        printf("FFT, index %d: %d, %d\n", fft_index, real, imag);
        fft_index ++;
      }
      break;

    case AVG:
      if (fft_index==fft_size) {
        if (word == 0) {
          state = WIN_HDR;
          if (VERBOSE) printf ("(End Frame)\n");
        } else {
          printf("ERROR: Expected delimiter after Average window, index: %d\n", fft_index);
          return -1;
        }
      } else {
        if (AVG_ZERO_CHK)
          if (word == 0){
            printf("ERROR: found zero in averages, index: %d\n", fft_index);
            return -1;
          }

        printf("Avg, index %d: %u\n", fft_index, word);
        fft_index ++;
      }
      break;

    case IDX_HDR:
      idx     = (unsigned short) (word & 0xFFFF);
      seq_num = (unsigned short) ((word>>16) & 0x3FFF);
      if (VERBOSE) printf("(FFT index header, seq num: %d)\n", seq_num);

      if (IDX_HDR_CHK)
        if (!(word&0x40000000)) {
          printf ("ERROR in FFT index %d, missing idx header bit.\n", idx);
          return -1;
        }

      if (idx >= fft_size) {
        printf ("ERROR in FFT index %d\n", idx);
        return -1;
      }
      fft_index = idx;
      state = FFT;
      break;

    default:
      return -1;
  }

  return 0;
}

// Finds index of the first proper window
unsigned int find_hdr(unsigned int* samples, unsigned int samples_len, unsigned int idx){
  int          after_zero = 0;
  unsigned int word;

  while (1){
    word = samples[idx];
    if ((after_zero) && (word & HDR_BIT))
      return idx;
    else
      after_zero = (word == 0);

    if (VERBOSE) printf("Trying to find a proper header\n");
    idx ++;
    if (idx == samples_len) return idx;
  }
}

int main( int argc, char *argv[] ) {
  FILE         *fp;
  unsigned int cur_buf_size;
  unsigned int cur_sample;

  int          parse_state;
  unsigned int left;
  unsigned int *averages = malloc (fft_size*sizeof(unsigned int));

  if (argc!=2){
    printf("Missing file name\n");
    return 0;
  }

  fp = fopen(argv[1],"rb");
  if (!fp){
    printf("Cannot open file\n");
    return 0;
  }

  cur_buf_size = fread(sample_buf, 4, buf_size, fp);
  if (cur_buf_size == 0){
    printf("Empty file\n");
    return 0;
  }
  // TODO: read in chunks until the end

  // Find first header
  cur_sample = find_hdr(sample_buf, cur_buf_size, 0);
  if (cur_sample == cur_buf_size){
    printf ("Could not find a proper window header\n");
    return 0;
  }

  state = WIN_HDR; // Going across files/buffers state can be carried over
  while (cur_sample < cur_buf_size){
    parse_state = parse_word(sample_buf[cur_sample]);

    if (parse_state == -1){
      cur_sample = find_hdr(sample_buf, cur_buf_size, cur_sample);
      if (cur_sample == cur_buf_size){
        printf ("Could not find a proper window header\n");
        return 0;
      }

    // Beginning of an average window, if condition can be commented out
    // as parser supports averages too and acts as no COPY_AVG
    } else if (parse_state == 1){
      cur_sample ++;
      if (cur_sample + fft_size > cur_buf_size)
        left = cur_buf_size - cur_sample;
      else
        left = fft_size;

      if (COPY_AVG){
        memcpy(averages, sample_buf+cur_sample, left*sizeof(unsigned int));
        printf("Copied Average window values.\n");
      } else {
        for (int i=0; i<left; i++)
          printf("Avg, index %d: %u\n", i, sample_buf[cur_sample+i]);
      }

      cur_sample += left;
      fft_index   = left;

    } else {
      cur_sample ++;
    }
  }

  return 0;
}
