#!/usr/bin/env ruby

=begin

Generates `[bands]` entries for ACARS channels

=end

# ACARS channel frequencies, in hertz
ACARS_CHANNELS = [
    129.125e6,
    129.350e6,
    129.525e6,
    130.025e6,
    130.450e6,
    130.825e6,
    131.125e6,
    131.425e6,
    131.550e6,
    131.725e6,
    131.850e6,
    136.575e6,
    136.650e6,
    136.675e6,
    136.700e6,
    136.725e6,
    136.750e6,
    136.775e6,
    136.800e6,
    136.850e6,
    136.975e6,
]
# Center frequency, hertz
CENTER = 131.0e6

ACARS_CHANNELS.each do |channel|
    human_friendly_channel = '%.3f' % (channel / 1e6)
    frequency_offset = channel - CENTER

    puts "
# #{human_friendly_channel} MHz
[[bands]]
bins = 2
frequency = #{frequency_offset}
destination = { type = \"file\", path = \"/home/samcrow/Documents/CurrentClasses/Research/Compression/acars_testing/multi_files/#{human_friendly_channel}.iq\"}"
end