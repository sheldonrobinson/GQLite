require 'tempfile'
require 'gqlite'
require 'yaml'

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
    r_node = /\((\w*)((:\w*)*)( {.*})?\)/
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

IgnoredScenario = [
  "[12] CREATE does not lose precision on large integers",
  "[19] Fail when adding new label predicate on a node that is already bound 5"
]

Before do |scenario|
  @ignored_scenario = IgnoredScenario.include? scenario.name
end

Given(/^any graph$/) do
  if @handle.nil?
    file = Tempfile.new('testdb')
    @handle = Gqlite::Database.new(sqlite_filename: file.path)
  end
end

Given(/^having executed:$/) do |string|
  next if @ignored_scenario
  @handle.execute_oc_query string
end

Given(/^an empty graph$/) do
  next if @ignored_scenario
  file = Tempfile.new('testdb')
  @handle = Gqlite::Database.new(sqlite_filename: file.path)
end

When(/^executing query:$/) do |string|
  next if @ignored_scenario
  begin
    @stats_before = GqliteTest.get_stats @handle
    @query_result = GqliteTest.parse_results(@handle.execute_oc_query string)
    @stats_after = GqliteTest.get_stats @handle
  rescue Gqlite::Error => exp
    @exception = exp
  end
end

When(/^executing control query:$/) do |string|
  next if @ignored_scenario
  begin
    @query_result = GqliteTest.parse_results(@handle.execute_oc_query string)
  rescue Gqlite::Error => exp
    @exception = exp
  end
end

Then(/^the side effects should be:$/) do |table|
  next if @ignored_scenario
  diff_stats = @stats_after .diff_to @stats_before
  ref_stats = GqliteTest.parse_side_effect_table table
  expect(diff_stats).to eq(ref_stats)
end

Then(/^the result should be empty$/) do
  next if @ignored_scenario
  expect(@query_result).to be_nil
end

Then(/^the result should be, in any order:$/) do |table|
  next if @ignored_scenario
  expect(@query_result).to eq(GqliteTest.parse_results_table table)
end

Then(/^a SyntaxError should be raised at compile time: VariableAlreadyBound$/) do
  next if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(\d+:\d+:|)Variable .* is already bound.$/)
end

Then(/^a SyntaxError should be raised at compile time: UndefinedVariable$/) do
  next if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^Variable .* is not defined.$/)
end

Then(/^a SyntaxError should be raised at compile time: NoSingleRelationshipType$/) do
  pending # Write code here that turns the phrase above into concrete actions
end

Then(/^a SyntaxError should be raised at compile time: RequiresDirectedRelationship$/) do
  pending # Write code here that turns the phrase above into concrete actions
end

Then(/^a SyntaxError should be raised at compile time: CreatingVarLength$/) do
  pending # Write code here that turns the phrase above into concrete actions
end
