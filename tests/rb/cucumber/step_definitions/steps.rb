require 'tempfile'
require 'gqlite'
require 'yaml'
require 'rspec'

class String
  def is_i?
     !!(self =~ /\A[-+]?[0-9]+\z/)
  end
end

class SideEffect
  attr_reader :nodes_count, :edges_count, :labels_count, :properties_count
  def initialize(nodes_count, edges_count, labels_count, properties_count)
    @nodes_count = nodes_count
    @edges_count = edges_count
    @labels_count = labels_count
    @properties_count = properties_count
  end
  def diff_to(rhs)
    return SideEffect.new(@nodes_count - rhs.nodes_count, @edges_count - rhs.edges_count, @labels_count - rhs.labels_count, @properties_count - rhs.properties_count)
  end
  def ==(rhs)
    @nodes_count == rhs.nodes_count and @edges_count == rhs.edges_count and @labels_count == rhs.labels_count and @properties_count == rhs.properties_count
  end
end

module GqliteTest
  module CApi
    extend FFI::Library
    ffi_lib 'gqlite'
    attach_function :gqlite_private_test_stats, [:pointer], :pointer
  end
  def GqliteTest.get_stats(handle)
    ret = CApi.gqlite_private_test_stats handle.dbhandle
    val = JSON.parse Gqlite::CApi.call_function :gqlite_value_to_json, ret
    Gqlite::CApi.call_function :gqlite_value_destroy, ret
    return SideEffect.new val["nodes_count"], val["edges_count"], val["labels_assignment_nodes_count"], val["properties_count"]
  end
  def GqliteTest.parse_side_effect_table(table)
    nodes_count = 0; edges_count = 0; labels_count = 0; properties_count = 0
    table.raw.each do |line|
      label = line[0]
      count = line[1].to_i
      if label == '+nodes'
        nodes_count = count
      end
      if label == '+relationships'
        edges_count = count
      end
      if label == '+labels'
        labels_count = count
      end
      if label == '+properties'
        properties_count = count
      end
    end
    return SideEffect.new nodes_count, edges_count, labels_count, properties_count
  end
  def GqliteTest.parse_results(results)
    return nil if results.nil?
    return results.map do |c|
      c.map { |v|
        if v.instance_of? Hash
          v.delete "id"
        end
        v
      }
    end
  end
  def GqliteTest.parse_results_table(table)
    table = table.raw
    r_node = /\((\w*)((:\w*)*)(\s*{.*})?\)/
    return table.map do |c|
      c.map { |v|
        if v == 'null'
          nil
        elsif v[0] == "'" and v[-1] == "'"
          v[1..-2]
        elsif v.is_i?
          v.to_i
        else
          arr = v.scan r_node
          if arr.size > 0
            labels = arr[0][1]
            if labels.nil?
              labels = []
            else
              labels = labels.split(":").reject(&:empty?)
            end
            properties = arr[0][3] 
            if properties.nil?
              properties = {}
            else
              properties = YAML.load properties
            end
            { "type"=>"node", "properties" => properties, "labels" => labels }
          else
            v
          end
        end
      }
    end
  end
end

RSpec::Matchers.matcher :eq_in_any_order do |expected|
  match do |actual|
    next false if actual.length != expected.length
    for i in 0..actual.length
      match = false
      for j in 0..actual.length
        if actual[i] == expected[j]
          match = true
          break
        end
      end
      unless match
        next false
      end
    end
    true
  end
end

IgnoredScenario = [
  # gqlite does not support large integer, as, sqlite does not, and added support for bignumber would add extra complexity withou a sqlite extension
  "[12] CREATE does not lose precision on large integers",
  # Triggers a different error first, as MATCH (a) return an empty list, it fails in creation
  # TODO revisit in case we add support for creation of relationship with list of nodes 
  "[24] Fail when creating a relationship using undefined variable in pattern",
  # Path assignment is not implemented yet
  # Nor is *
  "[8] Fail when a path has the same variable in a preceding MATCH",
  "[9] Fail when a relationship has the same variable in the same pattern",
  "[10] Fail when a path has the same variable in the same pattern",
  "[9] Fail when a node has the same variable in a preceding MATCH",
  "[10] Fail when a path has the same variable in a preceding MATCH",
  "[11] Fail when a node has the same variable in the same pattern",
  "[12] Fail when a path has the same variable in the same pattern",
  # p = ()-[]->() not supported (path assignment)
  "[12] Filter path with path length predicate on multi variables with one binding",
  "[13] Filter path with false path length predicate on multi variables with one binding",
  # with not implemented
  "[11] Fail when matching a node variable bound to a value",
  "[7] Matching twice with conflicting relationship types on same relationship",
  "[13] Fail when matching a relationship variable bound to a value",
  # bindings ($) not implemented
  "[8] Fail when using parameter as relationship predicate in MATCH",
  # <--> is considered an error, unclear if it is equivalent to -- (aka no specified direction,
  # to me it should indicate that there are two edges between the node, in each direction, but
  # is not the case according to the test case)
  "[19] Two bound nodes pointing to the same node",
  # Multi match statements are not supported yet
  "[20] Three bound nodes pointing to the same node",
  "[21] Three bound nodes pointing to the same node with extra connections",
  "[22] Returning bound nodes that are not part of the pattern",
  "[23] Matching disconnected patterns",
  "[24] Matching twice with duplicate relationship types on same relationship",
  "[25] Matching twice with an additional node label",
  "[26] Matching twice with a duplicate predicate",
  "[30] Fail when using a list or nodes as a node",
  # OPTIONAL MATCH is not supported yet
  "[27] Matching from null nodes should return no results owing to finding no matches",
  "[28] Matching from null nodes should return no results owing to matches being filtered out",
  # TODO edge isomorphism
  "[29] Fail when re-using a relationship in the same pattern",
  # paths are not supported in where
  "[2] Join with disjunctive multi-part predicates including patterns"
]

Before do |scenario|
  @ignored_scenario = false
  IgnoredScenario.each() do |ignored_scenario_name|
    if scenario.name.start_with?(ignored_scenario_name)
      @ignored_scenario = true
      break
    end
  end
end

Given(/^any graph$/) do
  if @handle.nil?
    file = Tempfile.new('testdb')
    @handle = Gqlite::Connection.new(sqlite_filename: file.path)
  end
end

Given(/^having executed:$/) do |string|
  pending if @ignored_scenario
  @handle.execute_oc_query string
end

Given(/^an empty graph$/) do
  pending if @ignored_scenario
  file = Tempfile.new('testdb')
  @handle = Gqlite::Connection.new(sqlite_filename: file.path)
end

When(/^executing query:$/) do |string|
  pending if @ignored_scenario
  begin
    @stats_before = GqliteTest.get_stats @handle
    @query_result = GqliteTest.parse_results(@handle.execute_oc_query string)
    @stats_after = GqliteTest.get_stats @handle
  rescue Gqlite::Error => exp
    @exception = exp
  end
end

When(/^executing control query:$/) do |string|
  pending if @ignored_scenario
  begin
    @query_result = GqliteTest.parse_results(@handle.execute_oc_query string)
  rescue Gqlite::Error => exp
    @exception = exp
  end
end

Then(/^the side effects should be:$/) do |table|
  pending if @ignored_scenario
  diff_stats = @stats_after .diff_to @stats_before
  ref_stats = GqliteTest.parse_side_effect_table table
  expect(diff_stats).to eq(ref_stats)
end

Then(/^no side effects$/) do
  pending if @ignored_scenario
  diff_stats = @stats_after .diff_to @stats_before
  expect(diff_stats).to eq(SideEffect.new 0, 0, 0, 0)
end

Then(/^the result should be empty$/) do
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).to be_nil
end

Then(/^the result should be, in any order:$/) do |table|
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).not_to be_nil
  puts @query_result
  expect(@query_result).to eq_in_any_order(GqliteTest.parse_results_table table)
end

Then(/^a SyntaxError should be raised at compile time: VariableAlreadyBound$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(\d+:\d+:|)Variable .* is already bound.|\d+:\d+:Expected token : got \]$/)
end

Then(/^a SyntaxError should be raised at compile time: UndefinedVariable$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^Variable .* is not defined.$/)
end

Then(/^a SyntaxError should be raised at compile time: NoSingleRelationshipType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(\d+:\d+:Expected token \[ got .*)|(\d+:\d+:Expected token \] got .*)$/)
end

Then(/^a SyntaxError should be raised at compile time: RequiresDirectedRelationship$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/(^\d+:\d+:Edge must be directed during creation.)|(\d+:\d+:Edge cannot have both direction.)$/)
end

Then(/^a SyntaxError should be raised at compile time: CreatingVarLength$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^\d+:\d+:Expected token \] got .*$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidParameterUse$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^\d+:\d+:Expected token \) got .*$/)
end

Then(/^a SyntaxError should be raised at compile time: VariableTypeConflict$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^.* is already bound.$/)
end

Then(/^a SyntaxError should be raised at compile time: RelationshipUniquenessViolation$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^.* is already bound.$/)
end
