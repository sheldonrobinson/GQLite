#!/usr/bin/env ruby

require 'cucumber'

if File.directory?('openCypher')
  last_update = File.open("openCypher_update").read.to_i
  if Time.now.to_i > (last_update + 60*60*24)
    puts "Updating openCypher"
    File.write("openCypher_update", Time.now.to_i)
    `cd openCypher; git pull`
  end
else
  puts "Cloning openCypher"
  `git clone https://github.com/opencypher/openCypher.git`
  File.write("openCypher_update", Time.now.to_i)
end

# Validated
features = [ 'create/Create1.feature', 'create/Create2.feature', 'match/Match1.feature' ]
# In progress
# features = [ 'create/Create5.feature' ]
# Current dev
# features = []

features = features.map { |file| 'openCypher/tck/features/clauses/' + file }
args = features.concat %w(--require cucumber/step_definitions/ --fail-fast)
# args = %w(cucumber/features)

begin
  Cucumber::Cli::Main.new(args).execute!
rescue SystemExit
  puts "Cucumber calls @kernel.exit(), killing your script unless you rescue"
end
