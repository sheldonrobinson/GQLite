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
features = [
  # Create
  'create/Create1.feature', 'create/Create2.feature',
  # Match
  'match/Match1.feature', 'match/Match2.feature', 'match/Match3.feature',
  # MatchWhere
  'match-where/MatchWhere1.feature', 'match-where/MatchWhere2.feature', 'match-where/MatchWhere3.feature', 'match-where/MatchWhere4.feature',
  # WITH
  'with/With1.feature', 'with/With2.feature', 'with/With3.feature', 'with/With4.feature',
  # DELETE
  'delete/Delete1.feature',
  # SET
  'set/Set1.feature', 'set/Set2.feature', 'set/Set3.feature', 'set/Set4.feature', 'set/Set5.feature',
  # REMOVE
  'remove/Remove1.feature', 'remove/Remove2.feature',
  # ORDER BY
  'return-orderby/ReturnOrderBy2.feature'
 ]
# In progress
# features = ['return-skip-limit/ReturnSkipLimit1.feature', 'create/Create5.feature', 'match/Match6.feature', 'match-where/MatchWhere5.feature', 'set/Set3.feature', 'set/Set5.feature', 'remove/Remove3.feature' ]
# Current dev
# features = []

features = features.map { |file| 'openCypher/tck/features/clauses/' + file }
args = features.concat %w(--require cucumber/step_definitions/ --fail-fast)

begin
  Cucumber::Cli::Main.new(args).execute!
rescue SystemExit
  puts "Cucumber calls @kernel.exit(), killing your script unless you rescue"
end
