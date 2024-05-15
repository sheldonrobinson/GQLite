#!/usr/bin/env ruby

require 'cucumber'
require 'optparse'

options = {
  :abort_on_first_error => false,
  :output_html => false,
  :output_junit => false
}

OptionParser.new do |opt|
  opt.on('--abort-on-first-error') { options[:abort_on_first_error] = true }
  opt.on('--output-html') { options[:output_html] = true }
  opt.on('--output-junit') { options[:output_junit] = true }
  opt.on('--output-dir OUT') { |o| options[:output_dir] = o}
end.parse!

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
  'create/Create1', 'create/Create2',
  # Match
  'match/Match1', 'match/Match2', 'match/Match3', 'match/Match7',
  # MatchWhere
  'match-where/MatchWhere1', 'match-where/MatchWhere2', 'match-where/MatchWhere3', 'match-where/MatchWhere4',
  # WITH
  'with/With1', 'with/With2', 'with/With3', 'with/With4',
  # DELETE
  'delete/Delete1', 'delete/Delete2',
  # SET
  'set/Set1', 'set/Set2', 'set/Set3', 'set/Set4', 'set/Set5',
  # REMOVE
  'remove/Remove1', 'remove/Remove2',
  # RETURN ORDER BY
  'return-orderby/ReturnOrderBy2', 'return-orderby/ReturnOrderBy3', 'return-orderby/ReturnOrderBy4',
  # RETURN SKIP LIMIT
  'return-skip-limit/ReturnSkipLimit1', 'return-skip-limit/ReturnSkipLimit2', 'return-skip-limit/ReturnSkipLimit3',
  # WITH ORDER BY
  'with-orderBy/WithOrderBy2', 'with-orderBy/WithOrderBy3', 'with-orderBy/WithOrderBy4',
  # WITH SKIP LIMIT
  'with-skip-limit/WithSkipLimit1', 'with-skip-limit/WithSkipLimit2', 'with-skip-limit/WithSkipLimit3',
  # UNWIND
  'unwind/Unwind1',
]

expressions = [
  # Aggregations
  'aggregation/Aggregation1', 'aggregation/Aggregation2', 'aggregation/Aggregation3', 'aggregation/Aggregation4', 'aggregation/Aggregation7',
  # boolean
  'boolean/Boolean1', 'boolean/Boolean2', 'boolean/Boolean3', 'boolean/Boolean4', 'boolean/Boolean5',
  # conditonal
  'conditional/Conditional1', 'conditional/Conditional2',
  # comparison
  'comparison/Comparison1', 'comparison/Comparison2', 'comparison/Comparison3', 'comparison/Comparison4',
  # graph
  'graph/Graph1', 'graph/Graph2', 'graph/Graph3', 'graph/Graph4', 'graph/Graph5', 'graph/Graph7', 'graph/Graph9',
  # list
  'list/List1', 'list/List2', 'list/List3', 'list/List4', 'list/List5', 
  # 'list/List6',
   'list/List7',  'list/List8', 
  #  'list/List9',
  # literals
  'literals/Literals1', 'literals/Literals2', 'literals/Literals3', 'literals/Literals4', 
  # null
  'null/Null1', 'null/Null2', 'null/Null3',
]

# In progress
# features = ['create/Create5', 'match/Match6', 'match-where/MatchWhere5', 'set/Set3', 'set/Set5', 'remove/Remove3', 'match/Match7' ]
# expressions = ['comparison/Comparison1', 'aggregation/Aggregation8']
# Current dev
# features = []
# expressions = []

# Build arguments

expressions = expressions.map { |file| 'openCypher/tck/features/expressions/' + file + '.feature' }
features = features.map { |file| 'openCypher/tck/features/clauses/' + file + '.feature' }

args = (features + expressions).concat %w(--require cucumber/step_definitions/)

args = args.concat ['--tags', '~@skipStyleCheck']
args = args.concat %w(--fail-fast)  if options[:abort_on_first_error]
args = args.concat %w(--format html)  if options[:output_html]
args = args.concat(%w(--format junit --out)).concat([options[:output_dir]])  if options[:output_junit]

# To st|o| op at first error
# args = (features + expressions).concat %w(--require cucumber/step_definitions/ --fail-fast)

# begin
Cucumber::Cli::Main.new(args).execute!
# rescue SystemExit => se
#   puts se.status
#   puts "Cucumber calls @kernel.exit(), killing your script unless you rescue"
# end
