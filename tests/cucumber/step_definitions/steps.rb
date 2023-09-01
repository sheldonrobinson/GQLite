require 'tempfile'
require 'gqlite'

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
    class Stats < FFI::Struct 
      layout :nodes_count, :int, 
             :edges_count, :int, 
             :labels_count, :int
    end
    attach_function :gqlite_private_test_stats, [:pointer], Stats
  end
  def GqliteTest.get_stats(handle)
    ret = CApi.gqlite_private_test_stats handle.dbhandle
    val = JSON.parse Gqlite::CApi.call_function :gqlite_value_to_json, ret
    Gqlite::CApi.call_function :gqlite_value_destroy, ret
    return SideEffect.new val["nodes_count"], val["edges_count"], val["labels_count"], val["properties_count"]
  end
  def GqliteTest.parse_side_effect_table(table)
    nodes_count = 0; edges_count = 0; labels_count = 0; properties_count = 0
    table.raw.each do |line|
      label = line[0]
      count = line[1].to_i
      if label == '+nodes'
        nodes_count = count
      end
      if label == '+edges'
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
end

Given(/^any graph$/) do
  if @handle.nil?
    file = Tempfile.new('testdb')
    @handle = Gqlite::Database.new(sqlite_filename: file.path)
    @current_stats = GqliteTest.get_stats @handle
  end
end

When(/^executing query:$/) do |string|
  @query_result = @handle.execute_oc_query string
end

Then(/^the result should be empty$/) do
  expect(@query_result).to be_nil
end

Then(/^the side effects should be:$/) do |table|
  new_current_stats = GqliteTest.get_stats @handle
  update = new_current_stats.diff_to @current_stats
  ref_stats = GqliteTest.parse_side_effect_table table
  expect(update).to eq(ref_stats)
  @current_stats = new_current_stats
end

Given(/^an empty graph$/) do
  file = Tempfile.new('testdb')
  @handle = Gqlite::Database.new(sqlite_filename: file.path)
  @current_stats = GqliteTest.get_stats @handle
end

Then(/^the result should be, in any order:$/) do |table|
  # table is a Cucumber::MultilineArgument::DataTable
  pending # Write code here that turns the phrase above into concrete actions
end

Then(/^a SyntaxError should be raised at compile time: VariableAlreadyBound$/) do
  pending # Write code here that turns the phrase above into concrete actions
end

Then(/^a SyntaxError should be raised at compile time: UndefinedVariable$/) do
  pending # Write code here that turns the phrase above into concrete actions
end
