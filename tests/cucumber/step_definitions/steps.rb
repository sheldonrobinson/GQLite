require 'tempfile'
require 'gqlite'

Given(/^any graph$/) do
  if @handle.nil?
    file = Tempfile.new('testdb')
    @handle = Gqlite::Database.new(sqlite_filename: file.path)
  end
end

When(/^executing query:$/) do |string|
  @query_result = @handle.execute_oc_query string
end

Then(/^the result should be empty$/) do
  expect(@query_result).to be_nil
end

Then(/^the side effects should be:$/) do |table|
  # table is a Cucumber::MultilineArgument::DataTable
  pending # Write code here that turns the phrase above into concrete actions
end

Given(/^an empty graph$/) do
  pending # Write code here that turns the phrase above into concrete actions
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
