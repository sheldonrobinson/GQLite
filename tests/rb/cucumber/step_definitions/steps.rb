require 'tempfile'
require 'gqlite'
require 'yaml'
require 'rspec'

require_relative 'comparison'

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

module GQLiteTest
  def GQLiteTest.get_stats(handle)
    val = handle.execute_oc_query "CALL gqlite.internal.stats()"
    return SideEffect.new val["nodes_count"], val["edges_count"], val["labels_assignment_nodes_count"], val["properties_count"]
  end
  def GQLiteTest.parse_side_effect_table(table)
    nodes_count = 0; edges_count = 0; labels_count = 0; properties_count = 0
    table.raw.each do |line|
      label = line[0]
      count = line[1].to_i
      if label == '+nodes'
        nodes_count += count
      end
      if label == '+relationships'
        edges_count += count
      end
      if label == '+labels'
        labels_count += count
      end
      if label == '+properties'
        properties_count += count
      end
      if label == '-nodes'
        nodes_count -= count
      end
      if label == '-relationships'
        edges_count -= count
      end
      if label == '-labels'
        labels_count -= count
      end
      if label == '-properties'
        properties_count -= count
      end
    end
    return SideEffect.new nodes_count, edges_count, labels_count, properties_count
  end
  def GQLiteTest.parse_results(results)
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
  def GQLiteTest.parse_results_table(table)
    table = table.raw
    r_node = /^\((\w*)((:\w*)*)(\s*{.*})?\)$/
    r_edge = /^\[(\w*)(:(\w*))?(\s*{.*})?\]$/
    first_row = true
    return table.map do |c|
      if first_row
        first_row = false
        c
      else
        c.map do |v|
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
          elsif v != "[]" && (arr = v.scan(r_edge)).size > 0
            label = arr[0][2]
            properties = arr[0][3]
            unless label.nil? && properties.nil?
              if properties.nil?
                properties = {}
              else
                properties = YAML.load properties
              end
              { "type"=>"edge", "properties" => properties, "label" => label }
            else
              YAML.load v
            end
          else
            YAML.load v
          end
        end
      end
    end
  end
end

RSpec::Matchers.matcher :eq_in_any_order do |expected|
  match do |actual|
    compare_table_in_any_order(actual, expected)
  end
end

IgnoredScenario = [
  # Triggers a different error first, as MATCH (a) return an empty list, it fails in creation
  "[24] Fail when creating a relationship using undefined variable in pattern",
  # Path assignment is not implemented yet
  "[4] Forwarding a path variable",
  "[14] Fail when filtering path with property predicate",
  "[8] `labels()` failing on a path",
  "[5] Fail for `size()` on paths",
  "[3] Delete relationship with bidirectional matching",
  "[3] Comparing across types yields null, except numbers",
  "[14] Direction of traversed relationship is not significant for path equality, simple",
  # Path assignment and variable length path are not implemented
  "[8] Fail when a path has the same variable in a preceding MATCH",
  "[9] Fail when a relationship has the same variable in the same pattern",
  "[10] Fail when a path has the same variable in the same pattern",
  "[9] Fail when a node has the same variable in a preceding MATCH",
  "[10] Fail when a path has the same variable in a preceding MATCH",
  "[11] Fail when a node has the same variable in the same pattern",
  "[12] Fail when a path has the same variable in the same pattern",
  "[12] Variable length optional relationships",
  "[13] Variable length optional relationships with bound nodes",
  "[14] Variable length optional relationships with length predicates",
  "[15] Variable length patterns and nulls",
  "[20] Variable length optional relationships with bound nodes, no matches",
  # p = ()-[]->() not supported (named path)
  "[12] Filter path with path length predicate on multi variables with one binding",
  "[13] Filter path with false path length predicate on multi variables with one binding",
  "[16] Optionally matching named paths - null result",
  "[17] Optionally matching named paths - existing result",
  "[18] Named paths inside optional matches with node predicates",
  "[19] Optionally matching named paths with single and variable length patterns",
  # WITH not implemented
  "[11] Fail when matching a node variable bound to a value",
  "[7] Matching twice with conflicting relationship types on same relationship",
  "[24] Matching twice with duplicate relationship types on same relationship",
  "[25] Matching twice with an additional node label",
  "[26] Matching twice with a duplicate predicate",
  "[30] Fail when using a list or nodes as a node",
  # <--> is considered an error, unclear if it is equivalent to -- (aka no specified direction,
  # to me it should indicate that there are two edges between the node, in each direction, but
  # is not the case according to the test case)
  "[19] Two bound nodes pointing to the same node",
  # OPTIONAL MATCH is not supported yet
  "[27] Matching from null nodes should return no results owing to finding no matches",
  "[28] Matching from null nodes should return no results owing to matches being filtered out",
  "[5] Forwarding null",
  "[6] Forwarind a node variable possibly null",
  "[4] Delete on null node",
  "[5] Ignore null when deleting node",
  "[6] Detach delete on null node",
  "[8] Ignore null when setting property",
  "[5] Ignore null when setting properties using an overriding map",
  "[8] Ignore null when setting label",
  "[1] Ignore null when setting properties using an appending map",
  "[5] Ignore null when removing property from a node",
  "[6] Ignore null when removing property from a relationship",
  "[5] Ignore null when removing a node label",
  "[7] `labels()` on null node",
  "[3] `type()` on null relationship",
  "[4] `type()` on mixed null and non-null relationships",
  "[5] Label expression on null",
  "[2] Statically access a property of a optional non-null node",
  "[3] Statically access a property of a null node",
  "[3] `properties()` on null",
  "[2] Delete optionally matched relationship",
  "[4] Ignore null when deleting relationship",
  # TODO edge isomorphism
  "[29] Fail when re-using a relationship in the same pattern",
  # WITH allow to reuse variable even with considering edge isomorphism, need a better way to handle that, might have to be tracked by the parser
  "[3] Forwarding a relationship variable",
  # paths are not supported in where
  "[2] Join with disjunctive multi-part predicates including patterns",
  # Aggregations are not supported yet
  "[5] Fail when not aliasing expressions in WITH",
  "[7] Remove a missing node property",
  "[3] Sort on aggregated function",
  "[6] Count star should count everything in scope",
  "[7] Ordering with aggregation",
  "[11] Aggregates ordered by arithmetics",
  "[1] Sort on aggregate function and normal property",
  "[22] Sort by an expression that is only partially orderable on a non-distinct binding table, but used as a grouping key",
  "[23] Sort by an expression that is only partially orderable on a non-distinct binding table, but used in parts as a grouping key",
  "[2] Ordering and skipping on aggregate",
  "[4] Ordering and limiting on aggregate",
  "[11] Sort by an aggregate projection",
  "[12] Sort by an aliased aggregate projection",
  "[13] Fail on sorting by a non-projected aggregation on a variable",
  "[14] Fail on sorting by a non-projected aggregation on an expression",
  "[15] Sort by an aliased aggregate projection does allow subsequent matching",
  "[16] Handle constants and parameters inside an order by item which contains an aggregation expression",
  "[19] Fail if not projected variables are used inside an order by item which contains an aggregation expression",
  "[4] Unwinding a collected unwound expression",
  "[5] Unwinding a collected expression",
  "[12] Unwind does not remove variables from scope",
  "[6] Reusing variable names in WITH",
  "[17] Handle projected variables inside an order by item which contains an aggregation expression",
  "[18]  Handle projected property accesses inside an order by item which contains an aggregation expression",
  "[1] Number-typed integer comparison",
  "[2] Number-typed float comparison",
  "[3] Any-typed string comparison",
  "[4] Comparing nodes to nodes",
  "[29] Satisfies the open world assumption, relationships between same nodes",
  "[30] Satisfies the open world assumption, single relationship",
  "[31] Satisfies the open world assumption, relationships between different nodes",
  # ORDER BY not supported yet
  # "[1] Forwarding multiple node and relationship variables",
  # No validation of delete expression
  "[8] Failing when deleting a label",
  # Missing variables should be handled by the parser
  "[9] Failing when using undefined variable in SET",
  # Lists are not supported in expressions
  "[5] Adding a list property",
  "[6] Concatenate elements onto a list property",
  "[7] Concatenate elements in reverse onto a list property",
  "[10] Failing when setting a list of maps as a property",
  # List comparison are not supported
  "[4] Comparing lists",
  # Distinct is not supported
  "[4] Support sort and distinct",
  "[5] Support ordering by a property after being distinct-ified",
  "[9] Using aliased DISTINCT expression in ORDER BY",
  "[10] Returned columns do not change from using ORDER BY",
  "[13] Fail when sorting on variable removed by DISTINCT",
  "[24] Sort by an expression that is only partially orderable on a non-distinct binding table, but made distinct",
  "[1] Handle dependencies across WITH with SKIP",
  # UNWIND not supported yet
  "[1] ORDER BY of a column introduced in RETURN should return salient results in ascending order",
  "[3] SKIP with an expression that does not depend on variables",
  "[3] Limiting amount of rows when there are fewer left than the LIMIT argument",
  "[1] Limit to two hits",
  "[6] LIMIT with an expression that does not depend on variables",
  "[7] The order direction cannot be overwritten",
  # return modifiers (e.g. order by) must be done after computing expressions https://gitlab.com/gqlite/GQLite/-/issues/2
  "[7] Limit to more rows than actual results 1",
  "[8] Limit to more rows than actual results 2",
  "[15] Floating point parameter for LIMIT with ORDER BY should fail",
  # non-projected variable are not accessible
  "[21] Sort by an expression that is only partially orderable on a non-distinct binding table",
  "[8] Sort by non-projected existing variable",
  # Array aren't really supported yet
  "[9] Sort by a list expression in ascending order",
  "[10] Sort by a list expression in descending order",
  # Time not supported
  "[11] Sort by a date expression in ascending order",
  "[12] Sort by a date expression in descending order",
  "[13] Sort by a local time expression in ascending order",
  "[14] Sort by a local time expression in descending order",
  "[15] Sort by a time expression in ascending order",
  "[16] Sort by a time expression in descending order",
  "[17] Sort by a local date time expression in ascending order",
  "[18] Sort by a local date time expression in descending order",
  "[19] Sort by a date time expression in ascending order",
  "[20] Sort by a date time expression in descending order",
  # MERGE not supported
  "[6] Creating nodes from an unwound parameter list",
  "[14] Unwind with merge",
  # WITH WHERE not supported
  "[5] Conjunction is commutative on null",
  "[7] Conjunction is associative on null",
  "[5] Disjunction is commutative on null",
  "[7] Disjunction is associative on null",
  "[5] Exclusive disjunction is commutative on null",
  "[7] Exclusive disjunction is associative on null",
  "[2] Disjunction is distributive over conjunction on null",
  "[4] Conjunction is distributive over disjunction on null",
  "[6] Conjunction is not distributive over exclusive disjunction on null",
  # CASE WHEN THEN
  "[1] Simple cases over integers",
  # No compile time argument check
  "[7] Failing when using `type()` on a node",
  # pattern comprehension are not supported
  "[7] Using size of pattern comprehension to test existence",
  "[8] Get node degree via size of pattern comprehension",
  "[9] Get node degree via size of pattern comprehension that specifies a relationship type",
  "[10] Get node degree via size of pattern comprehension that specifies multiple relationship types",
  # handling of null
  "[4] Equality between almost equal lists with null should return null",
  "[7] Equality between almost equal nested lists with null should return null",
  "[6] Comparing lists to lists, Examples (#2)",
  "[6] Comparing lists to lists, Examples (#5)",
  "[7] Comparing maps to maps, Examples (#12)",
  "[7] Comparing maps to maps, Examples (#13)",
  "[7] Comparing maps to maps, Examples (#14)",
  "[7] Comparing maps to maps, Examples (#15)",
  "[7] Comparing maps to maps, Examples (#16)",
  # "[21] IN should return null if LHS and RHS are null - list version",
  "[29] IN should return null if comparison with null is required, list version",
  # "[31] IN should return null when comparing two so-called identical lists where one element is null",
  # "[34] IN should return null if comparison with null is required, list version 2",
  # variable inside the same CREATE statement are not accessible yet (https://gitlab.com/gqlite/GQLite/-/issues/9)
  "[1] Forwarding a property to express a join",
  "[2] Handle dependencies across WITH with LIMIT",
  # type of value stored in array/map is lost
  "[5] `type()` handling Any type",
  # Comparison
  "[5] Comparing relationships to relationships",
  "[5] Comparing NaN",
  "[6] Comparability between numbers and strings",
  "[8] Equality and inequality of NaN",
  # NULL and OPTIONAL
  "[3] Property null check on null node",
  "[3] Property not null check on null node"
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
    @handle = GQLite::Connection.new(sqlite_filename: file.path)
  end
end

Given(/^having executed:$/) do |string|
  pending if @ignored_scenario
  @handle.execute_oc_query string
end

Given(/^an empty graph$/) do
  pending if @ignored_scenario
  file = Tempfile.new('testdb')
  @handle = GQLite::Connection.new(sqlite_filename: file.path)
end

Given(/^parameters are:$/) do |table|
  @bindings = {}
  table.raw.each() do |row|
    null = nil
    @bindings["$" + row[0]] = eval(row[1])
  end
end

When(/^executing query:$/) do |string|
  pending if @ignored_scenario
  begin
    @stats_before = GQLiteTest.get_stats @handle
    @query_result = GQLiteTest.parse_results(@handle.execute_oc_query string, bindings: @bindings)
    @stats_after = GQLiteTest.get_stats @handle
  rescue GQLite::Error => exp
    @exception = exp
  end
end

When(/^executing control query:$/) do |string|
  pending if @ignored_scenario
  begin
    @query_result = GQLiteTest.parse_results(@handle.execute_oc_query string)
  rescue GQLite::Error => exp
    @exception = exp
  end
end

Then(/^the side effects should be:$/) do |table|
  pending if @ignored_scenario
  diff_stats = @stats_after .diff_to @stats_before
  ref_stats = GQLiteTest.parse_side_effect_table table
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
  expect(@query_result).to eq_in_any_order(GQLiteTest.parse_results_table table)
end

Then(/^the result should be, in order:$/) do |table|
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).not_to be_nil
  expect(@query_result).to eq(GQLiteTest.parse_results_table table)
end

Then(/^a SyntaxError should be raised at compile time: VariableAlreadyBound$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  # TODO: should not match for RunTime
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): (VariableAlreadyBound: Variable .* is already bound( at \(\d+, \d+\))?\.)|(UnexpectedSyntax: Expected token .* got .* at \(\d+, \d+\)\.)$/)
end

Then(/^a SyntaxError should be raised at compile time: UndefinedVariable$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  # TODO: should not match for RunTime
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): UndefinedVariable: Unknown variable '.*'\.$/)
end

Then(/^a SyntaxError should be raised at compile time: NoSingleRelationshipType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: .* at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: RequiresDirectedRelationship$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: RequiresDirectedRelationship: (Edge cannot have both direction)|(Edge must be directed during creation) at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: CreatingVarLength$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: Expected token .* got .* at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidParameterUse$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: InvalidParameterUse: Unknown parameter '\$.*' at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: VariableTypeConflict$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: VariableTypeConflict: .* is redefined as a variable of a different type.$/)
end

Then(/^a SyntaxError should be raised at compile time: RelationshipUniquenessViolation$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^.* is already bound.$/)
end

Then(/^a SyntaxError should be raised at compile time: ColumnNameConflict$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: ColumnNameConflict: Duplicate column name .* is not allowed at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: NoExpressionAlias$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^\d+:\d+:Expected token AS got {}.$/)
end

Then(/^a ConstraintVerificationFailed should be raised at runtime: DeleteConnectedNode$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^Cannot delete node with \d+ relationships.$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidDelete$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: InvalidDelete: .*\.$/)
end

Then(/^a SyntaxError should be raised at compile time: NonConstantExpression$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: NonConstantExpression: .*\.$/)
end

Then(/^an ArgumentError should be raised at runtime: NegativeIntegerArgument$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  # SKIP negative arguments with bindings are checked at CompileTime
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): NegativeIntegerArgument: .*\.$/)
end

Then(/^a SyntaxError should be raised at compile time: NegativeIntegerArgument$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): NegativeIntegerArgument: .*$/)
end

Then(/^a ArgumentError should be raised at runtime: InvalidArgumentType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): InvalidArgumentType: .*$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidArgumentType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: InvalidArgumentType: .*$/)
end

Then(/^a TypeError should be raised at any time: InvalidArgumentType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(CompileTime)|(RunTime): InvalidArgumentType: .*$/)
end

Then(/^a TypeError should be raised at any time: \*$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^(CompileTime)|(RunTime): InvalidArgumentType: .*$/)
end

Then(/^a SyntaxError should be raised at compile time: IntegerOverflow$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: IntegerOverflow: .*\.$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidNumberLiteral$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: (InvalidNumberLiteral.*)|(.*Expected token end of file got identifier.*)$/)
end

Then(/^a SyntaxError should be raised at compile time: UnexpectedSyntax$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: .* \(\d+, \d+\)\.$/)
end
