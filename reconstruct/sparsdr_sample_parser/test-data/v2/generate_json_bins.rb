#!/usr/bin/env ruby


literals = (0...1024).map do |i|
    if i == 92
        '[48, 55]'
    elsif i == 93
        '[21, -47]'
    elsif i == 94
        '[11, 51]'
    elsif i == 95
        '[-32, 64]'
    else
        '[0, 0]'
    end
end

puts literals.join(",\n")
