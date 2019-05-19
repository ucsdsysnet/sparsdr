#!/usr/bin/env python

"""
Generates GNU Radio Companion block XML for the SparSDR reconstruct block
"""

# Maximum number of decompression bands to support
MAX_BANDS = 32
DOCUMENTATION = """
The SparSDR reconstruct block receives compressed samples and reconstructs \
signals from one or more bands.

Parameters:

bands: The number of bands to reconstruct

Band parameters for band i:

Band i frequency: The center frequency to reconstruct, in herz relative to the \
center frequency used to capture the signals

Band i bins: The number of bins to use when reconstructing. This determines \
the bandwidth to reconstruct and the sample rate of the resulting signal.

"""

def generate_reconstruct_xml():
    return """<?xml version="1.0"?>
<block>
  <name>SparSDR Reconstruct</name>
  <!-- The variable_ prefix is required to generate code from the var_make
  template -->
  <key>variable_sparsdr_reconstruct</key>
  <category>[SparSDR]</category>
  <import>import sparsdr</import>

  <var_make>
{make_code}
  </var_make>
  <!-- Use var_make instead -->
  <make></make>

  <!-- Path to the sparsdr_reconstruct executable -->
  <param>
    <name>Executable</name>
    <key>reconstruct_path</key>
    <value>sparsdr_reconstruct</value>
    <type>string</type>
  </param>
  <!-- Number of bands -->
  <param>
    <name>Bands</name>
    <key>band_count</key>
    <value>1</value>
    <type>int</type>
  </param>

  {band_params}

  <check>{max_bands} >= $band_count</check>
  <check>$band_count > 0</check>

  <sink>
    <name>in</name>
    <!-- Matches output of uhd_usrp_source when set up correctly -->
    <type>sc16</type>
  </sink>

  <source>
    <name>out</name>
    <type>complex</type>
    <nports>$band_count</nports>
  </source>
  <doc>
{documentation}
  </doc>
</block>
    """.format(
        make_code = generate_make_code(),
        band_params = generate_band_params(),
        documentation = DOCUMENTATION,
        max_bands = MAX_BANDS,
    )

def generate_make_code():
    code = '$(id)_bands = sparsdr.band_spec_vector()\n'
    for i in range(MAX_BANDS):
        code += """#if $band_count() > {i}
$(id)_bands.push_back(sparsdr.band_spec($band_{i}_frequency, $band_{i}_bins))
#end if
""".format(i=i)
    code += 'self.$(id) = $(id) = sparsdr.reconstruct(bands=$(id)_bands, reconstruct_path=$reconstruct_path)\n'
    return code

def generate_band_params():
    params = ''
    for i in range(MAX_BANDS):
        params += """
  <param>
    <name>Band {i} frequency</name>
    <key>band_{i}_frequency</key>
    <value>0.0</value>
    <type>real</type>
		<hide>#if $band_count() > {i} then 'none' else 'all'#</hide>
		<tab>Bands</tab>
  </param>
  <param>
    <name>Band {i} bins</name>
    <key>band_{i}_bins</key>
    <value>2048</value>
    <type>int</type>
		<hide>#if $band_count() > {i} then 'none' else 'all'#</hide>
		<tab>Bands</tab>
  </param>
        """.format(i=i)
    return params

if __name__ == "__main__":
    print(generate_reconstruct_xml())
