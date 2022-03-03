#!/usr/bin/env ruby

TEMPLATE = <<EOT
  <param>
    <name>Band %{n} frequency</name>
    <key>band_%{n}_frequency</key>
    <value>2.426e9</value>
    <type>real</type>
    <hide>#if $band_count() > %{n} then 'none' else 'all'#</hide>
    <tab>Bands</tab>
  </param>
  <param>
    <name>Band %{n} bandwidth</name>
    <key>band_%{n}_bandwidth</key>
    <value>2e6</value>
    <type>real</type>
    <hide>#if $band_count() > %{n} then 'none' else 'all'#</hide>
    <tab>Bands</tab>
  </param>
EOT

(0...32).each do |n|
    puts TEMPLATE % {n: n}
end
