#!/usr/bin/env ruby

require 'cucumber'
require 'optparse'
require_relative 'cucumber/step_definitions/global'

options = {
  :abort_on_first_error => false,
  :output_html => false,
  :output_junit => false,
  :backend => "sqlite"
}

OptionParser.new do |opt|
  opt.on('--abort-on-first-error') { options[:abort_on_first_error] = true }
  opt.on('--output-html') { options[:output_html] = true }
  opt.on('--output-junit') { options[:output_junit] = true }
  opt.on('--output-dir OUT') { |o| options[:output_dir] = o}
  opt.on('--backend BACKEND') { |o| options[:backend] = o}
end.parse!

if File.directory?('openCypher')
  last_update = File.open("openCypher_update").read.to_i
  if Time.now.to_i > (last_update + 60*60*24)
    puts "Updating openCypher"
    File.write("openCypher_update", Time.now.to_i)
    `cd openCypher; git pull`
  end
  `cd openCypher; git checkout main`
else
  puts "Cloning openCypher"
  `git clone https://github.com/opencypher/openCypher.git`
  File.write("openCypher_update", Time.now.to_i)
end

# GQLite 2.x

gql = [
  # Graph
  'Graph1', 'Graph2'
  ]

extras = [
  # id
  'Id'
]

features = [
  # Create
  'create/Create1', 'create/Create2', 'create/Create3',
  # Match
  'match/Match1', 'match/Match2', 'match/Match3', 'match/Match7',
  # MatchWhere
  'match-where/MatchWhere1', 'match-where/MatchWhere2', 'match-where/MatchWhere3', 'match-where/MatchWhere4', 'match-where/MatchWhere5',
  # WITH
  'with/With1', 'with/With2',  'with/With4',
  # DELETE
  'delete/Delete1', 'delete/Delete2',
  # SET
  'set/Set1', 'set/Set2', 'set/Set3', 'set/Set4', 'set/Set5',
  # REMOVE
  'remove/Remove1', 'remove/Remove2', 'remove/Remove3',
  # UNWIND
  'unwind/Unwind1',
  # RETURN
  'return/Return1', 'return/Return2', 'return/Return3', 'return/Return4', 'return/Return6', 'return/Return7',  'return/Return8',
  # RETURN ORDER BY
  'return-orderby/ReturnOrderBy1', 'return-orderby/ReturnOrderBy2', 'return-orderby/ReturnOrderBy3', 'return-orderby/ReturnOrderBy4', 'return-orderby/ReturnOrderBy5',
  # RETURN SKIP LIMIT
  'return-skip-limit/ReturnSkipLimit1', 'return-skip-limit/ReturnSkipLimit2', 'return-skip-limit/ReturnSkipLimit3',
  # WITH
  'with/With1', 'with/With2', 'with/With3', 'with/With4', 'with/With6', 'with/With7',
  # WITH ORDER BY
  'with-orderBy/WithOrderBy1', 'with-orderBy/WithOrderBy2', 'with-orderBy/WithOrderBy3', 'with-orderBy/WithOrderBy4',
  # WITH SKIP LIMIT
  'with-skip-limit/WithSkipLimit1', 'with-skip-limit/WithSkipLimit2', 'with-skip-limit/WithSkipLimit3',
  # with-where
  'with-where/WithWhere1', 'with-where/WithWhere2', 'with-where/WithWhere3', 'with-where/WithWhere4', 'with-where/WithWhere5', 'with-where/WithWhere6', 'with-where/WithWhere7',
]

expressions = [
  # Aggregations
  'aggregation/Aggregation1', 'aggregation/Aggregation2', 'aggregation/Aggregation3', 'aggregation/Aggregation4', 'aggregation/Aggregation7',
  # Boolean
  'boolean/Boolean1', 'boolean/Boolean2', 'boolean/Boolean3', 'boolean/Boolean4', 'boolean/Boolean5',
  # Comparison
  'comparison/Comparison1', 'comparison/Comparison2',
  # Conditonal
  'conditional/Conditional1', 'conditional/Conditional2',
  # Null
  'null/Null1', 'null/Null2', 'null/Null3',
  # Graph
  'graph/Graph1', 'graph/Graph2', 'graph/Graph3', 'graph/Graph4', 'graph/Graph5', 'graph/Graph6', 'graph/Graph7', 'graph/Graph9',
  # List
  'list/List1', 'list/List2', 'list/List3', 'list/List4', 'list/List5', 'list/List6', 'list/List7',  'list/List8', 
  # Literals
  'literals/Literals1', 'literals/Literals2', 'literals/Literals3', 'literals/Literals4', 'literals/Literals5', 'literals/Literals6', 'literals/Literals7', 'literals/Literals8', 
  # Map
  'map/Map1', 'map/Map2', 'map/Map3', 
]

# In progress
# gql = []
# features = [
# ]
# expressions = [
# ]
# # Current dev
# gql = []
# features = [ ]
# expressions = [
# ]

# In progress
# gql = []
# features = ['create/Create4', 'create/Create6', 'create/Create5', 'match/Match6', 'remove/Remove3', 'return/Return5', 'with/With5', ]
# expressions = [ 'aggregation/Aggregation8']

# Regressions 1.1 -> 1.2

# expressions = [
#   # graph, those are parser problems, and will wait for the new nom parser
#  'comparison/Comparison3' 'comparison/Comparison4',
# ]

# Build arguments

gql = gql.map { |file| 'gql/features/' + file + '.feature' }
extras = extras.map { |file| 'extras/' + file + '.feature' }
expressions = expressions.map { |file| 'openCypher/tck/features/expressions/' + file + '.feature' }
features = features.map { |file| 'openCypher/tck/features/clauses/' + file + '.feature' }

args = (gql + features + expressions + extras).concat %w(--require cucumber/step_definitions/)

args = args.concat ['--tags', '~@skipStyleCheck']
args = args.concat %w(--fail-fast)  if options[:abort_on_first_error]
args = args.concat %w(--format html)  if options[:output_html]
args = args.concat(%w(--format junit --out)).concat([options[:output_dir]])  if options[:output_junit]

# To st|o| op at first error
# args = (features + expressions).concat %w(--require cucumber/step_definitions/ --fail-fast)

Global.store_backend = options[:backend]
begin
  Cucumber::Cli::Main.new(args).execute!
rescue SystemExit => se
  if se.status != 0
    exit(se.status)
  end
end
