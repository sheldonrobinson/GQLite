#!/usr/bin/env ruby
require 'csv'

if ARGV.length != 1
  STDERR.puts "Usage: #{$0} <output_prefix>"
  exit 1
end

prefix = ARGV[0]
input = $stdin.read

# Find start of Divan output (look for 'fastest' to identify the benchmark table)
bench_start = input.index("fastest")
if bench_start.nil?
  STDERR.puts "❌ No Divan benchmark output found in input."
  exit 1
end

divan_output = input[bench_start..]
lines = divan_output.lines.map(&:chomp)

# Save raw benchmark block to prefix.txt
txt_file = File.open("#{prefix}.txt", "w")

bench_data = []
current_group = nil

regex = /
  ^                               
  │?\s*                            # optional pipe and leading whitespace
  [├╰]                              # box drawing character
  \s*─\s*                           # optional dashes and spacing
  (\w+)                               # backend name
  \s+
  ([\d.]+\s*(?:ms|s))\s*│\s*        # fastest
  ([\d.]+\s*(?:ms|s))\s*│\s*        # slowest
  ([\d.]+\s*(?:ms|s))\s*│\s*        # median
  ([\d.]+\s*(?:ms|s))\s*│\s*        # mean
  (\d+)\s*│\s*                      # samples
  (\d+)                             # iters
/x

lines.each do |line|
  case line
  when /^fastest\s*│\s*slowest\s*│\s*median\s*│\s*mean\s*│\s*samples\s*│\s*iters/
    txt_file.puts(line)
  when /^[├╰]─\s*([a-zA-Z0-9_]+)\s+│/
    current_group = $1
    txt_file.puts(line)
  when regex
    backend, fastest, slowest, median, mean, samples, iters = $1, $2, $3, $4, $5, $6, $7
    bench_data << [current_group, backend, fastest, slowest, median, mean, samples, iters]
    txt_file.puts(line)
  end
end

# Output to CSV
CSV.open("#{prefix}.csv", "w", headers: ["group", "backend", "fastest", "slowest", "median", "mean", "samples", "iters"], write_headers: true) do |csv|
  bench_data.each { |row| csv << row }
end
