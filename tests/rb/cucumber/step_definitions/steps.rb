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
    return SideEffect.new val["nodes_count"], val["edges_count"], val["labels_nodes_count"], val["properties_count"]
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
    r_node = /^\(((\:\w*)*)(\s*{.*})?\)$/
    r_edge = /^\[((\:\w*)*)(\s*{.*})?\]$/
    r_path = /^\<\(((\:\w*)*)(\s*{.*})?\)\-\[((\:\w*)*)(\s*{.*})?\]\-\>\(((\:\w*)*)(\s*{.*})?\)\>$/
    first_row = true
    return table.map do |c|
      if first_row
        first_row = false
        c
      else
        c.map do |v|
          arr = v.scan r_node
          if arr.size > 0
            labels = arr[0][0]
            if labels.nil?
              labels = []
            else
              labels = labels.split(":").reject(&:empty?)
            end
            properties = arr[0][2] 
            if properties.nil?
              properties = {}
            else
              properties = YAML.load properties
            end
            { "type"=>"node", "properties" => properties, "labels" => labels }
          elsif v != "[]" && (arr = v.scan(r_edge)).size > 0
            labels = arr[0][0]
            properties = arr[0][2]
            unless labels.nil? && properties.nil?
              if labels.nil?
                labels = []
              else
                labels = labels.split(":").reject(&:empty?)
              end
              if properties.nil?
                properties = {}
              else
                properties = YAML.load properties
              end
              { "type"=>"edge", "properties" => properties, "labels" => labels }
            else
              YAML.load v
            end
          elsif (arr = v.scan(r_path)).size > 0
            # Parse source
            source_labels = arr[0][0]
            if source_labels.nil?
              source_labels = []
            else
              source_labels = source_labels.split(":").reject(&:empty?)
            end
            source_properties = arr[0][2] 
            if source_properties.nil?
              source_properties = {}
            else
              source_properties = YAML.load source_properties
            end
            # Parse edge
            edge_labels = arr[0][3]
            if edge_labels.nil?
              edge_labels = []
            else
              edge_labels = edge_labels.split(":").reject(&:empty?)
            end
            edge_properties = arr[0][5] 
            if edge_properties.nil?
              edge_properties = {}
            else
              edge_properties = YAML.load edge_properties
            end
            # Parse destination
            destination_labels = arr[0][6]
            if destination_labels.nil?
              destination_labels = []
            else
              destination_labels = destination_labels.split(":").reject(&:empty?)
            end
            destination_properties = arr[0][8] 
            if destination_properties.nil?
              destination_properties = {}
            else
              destination_properties = YAML.load destination_properties
            end
            {
              "type"=>"path",
              "source" => { "type"=>"node", "properties" => source_properties, "labels" => source_labels },
              "properties" => edge_properties, "labels" => edge_labels,
              "destination" => { "type"=>"node", "properties" => destination_properties, "labels" => destination_labels }
            }
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

RSpec::Matchers.matcher :eq_in_order do |expected|
  match do |actual|
    compare_table_in_order(actual, expected)
  end
end

IgnoredScenario = [
  # Regressions for gqlite 1.2:
  # aggregation not supported in ORDER BY
  "[3] Sort on aggregated function",
  "[6] Count star should count everything in scope",
  "[7] Ordering with aggregation",
  "[14] Fail on aggregation in ORDER BY after RETURN",
  "[1] Sort on aggregate function and normal property",
  "[4] Ordering and limiting on aggregate",
  # Missing dates functions
  "[11] Sort dates in ascending order",
  "[12] Sort dates in descending order",
  "[13] Sort local times in ascending order",
  "[14] Sort local times in descending order",
  "[15] Sort times in ascending order",
  "[16] Sort times in descending order",
  "[17] Sort local date times in ascending order",
  "[18] Sort local date times in descending order",
  "[19] Sort date times in ascending order",
  "[20] Sort date times in descending order",
  "[33] Sort by a date variable projected from a node property in ascending order",
  "[34] Sort by a date variable projected from a node property in descending order",
  "[35] Sort by a local time variable projected from a node property in ascending order",
  "[36] Sort by a local time variable projected from a node property in descending order",
  "[37] Sort by a time variable projected from a node property in ascending order",
  "[38] Sort by a time variable projected from a node property in descending order",
  "[39] Sort by a local date time variable projected from a node property in ascending order",
  "[40] Sort by a local date time variable projected from a node property in descending order",
  "[41] Sort by a date time variable projected from a node property in ascending order",
  "[42] Sort by a date time variable projected from a node property in descending order",
  "[43] Sort by a variable that is only partially orderable on a non-distinct binding table",
  "[44] Sort by a variable that is only partially orderable on a non-distinct binding table, but made distinct",
  # Missing min function
  "[13] Fail on sorting by a non-projected aggregation on a variable",
  "[14] Fail on sorting by a non-projected aggregation on an expression",

  # Not currently supported:
  "[5] Match relationship with inline property value",
  # Triggers a different error first, as MATCH (a) return an empty list, it fails in creation
  "[24] Fail when creating a relationship using undefined variable in pattern",
  # Variable lenght not supported
  "[22] Fail when creating a variable-length relationship",
  "[14] Fail when filtering path with property predicate",
  # Only basic pasth assignment supported yet
  "[4] Forwarding a path variable",
  # # Path assignment is not implemented yet
  # "[8] `labels()` failing on a path",
  # "[5] Fail for `size()` on paths",
  # "[3] Delete relationship with bidirectional matching",
  # "[3] Comparing across types yields null, except numbers",
  # "[14] Direction of traversed relationship is not significant for path equality, simple",
  # # p = ()-[]->() not supported (named path)
  # "[16] Optionally matching named paths - null result",
  # "[17] Optionally matching named paths - existing result",
  # "[18] Named paths inside optional matches with node predicates",
  # "[9] `labels()` failing on invalid arguments",
  # "[6] `type()` failing on invalid arguments",
  # paths are not supported in where
  "[2] Join with disjunctive multi-part predicates including patterns",
  # Path assignment and variable length path are not implemented
  "[12] Aggregation of named paths",
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
  "[19] Optionally matching named paths with single and variable length patterns",
  "[20] Variable length optional relationships with bound nodes, no matches",
  # WITH not eliminating conflicts
  "[7] Matching twice with conflicting relationship types on same relationship",
  "[24] Matching twice with duplicate relationship types on same relationship",
  "[25] Matching twice with an additional node label",
  "[26] Matching twice with a duplicate predicate",
  # <--> is considered an error, unclear if it is equivalent to -- (aka no specified direction,
  # to me it should indicate that there are two edges between the node, in each direction, but
  # is not the case according to the test case)
  "[19] Two bound nodes pointing to the same node",
  # OPTIONAL MATCH is not supported yet
  "[3] `type()` on null relationship",
  "[4] `type()` on mixed null and non-null relationships",
  "[5] Label expression on null",
  # TODO edge isomorphism
  "[29] Fail when re-using a relationship in the same pattern",
  # WITH allow to reuse variable even with considering edge isomorphism, need a better way to handle that, might have to be tracked by the parser
  "[3] Forwarding a relationship variable",
  # Unreported error
  "[5] Fail when not aliasing expressions in WITH",
  # aggregation (or function call) fail to access a variable when used in a WITH statement (such as ´count(you.age)´) https://gitlab.com/GQLite/GQLite/-/issues/16
  "[11] Sort by an aggregate projection",
  "[12] Sort by an aliased aggregate projection",
  "[16] Handle constants and parameters inside an order by item which contains an aggregation expression",
  "[17] Handle projected variables inside an order by item which contains an aggregation expression",
  "[18]  Handle projected property accesses inside an order by item which contains an aggregation expression",
  "[6] Reusing variable names in WITH",
  "[22] Sort by an expression that is only partially orderable on a non-distinct binding table, but used as a grouping key",
  "[23] Sort by an expression that is only partially orderable on a non-distinct binding table, but used in parts as a grouping key",
  "[15] Sort by an aliased aggregate projection does allow subsequent matching",
  # Missing RANGE function
  "[4] Unwinding a collected unwound expression",
  # GROUP BY fails with WITH statement https://gitlab.com/GQLite/GQLite/-/issues/17
  "[2] Ordering and skipping on aggregate",
  # collect(nodes) returns an array with the node internal id as an integer (aka it loses the information that it is a ref)
  "[5] Unwinding a collected expression",
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
  "[12] Unwind does not remove variables from scope",
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
  "[11] WITH-MERGE-CREATE: A bound node should be recognized after projection with WITH + MERGE node",
  "[12] WITH-MERGE-CREATE: A bound node should be recognized after projection with WITH + MERGE pattern",
  "[13] Merge followed by multiple creates",
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
  "[3] `properties()` on null",
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
  "[3] Property not null check on null node",
  # MAX aggregation on list
  "[9] `max()` over list values", # Needs a custom max/min aggregator see https://gitlab.com/GQLite/GQLite/-/issues/12
  "[11] `max()` over mixed values",
  "[12] `min()` over mixed values",
  # Correctly fail, but with different error message:
  "[25] Fail on sorting by an aggregation",
  "[19] Fail if not projected variables are used inside an order by item which contains an aggregation expression",
  "[20] Fail if more complex expressions, even if projected, are used inside an order by item which contains an aggregation expression",
  # No checking of index type
  "[8] Fail when indexing with a non-integer",
  "[9] Fail when indexing with a non-integer given by a parameter",
  # Multi-edge not supported yet, aka ()-[]-()-[]-()
  "[7] Fail when a relationship has the same variable in a preceding MATCH",
  # [x IN values] not supported
  "[45] Sort order should be consistent with comparisons where comparisons are defined #Example: <exampleName>",

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
    path = file.path
    file.unlink
    @handle = GQLite::Connection.new(filename: path)
  end
end

Given(/^having executed:$/) do |string|
  pending if @ignored_scenario
  @handle.execute_oc_query string
end

Given(/^an empty graph$/) do
  pending if @ignored_scenario
  file = Tempfile.new('testdb')
  path = file.path
  file.unlink
  @handle = GQLite::Connection.new(filename: path)
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
  puts @query_result
  puts GQLiteTest.parse_results_table table
  expect(@query_result).to eq_in_any_order(GQLiteTest.parse_results_table table)
end

Then(/^the result should be, in order:$/) do |table|
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).not_to be_nil
  expect(@query_result).to eq_in_order(GQLiteTest.parse_results_table table)
end

Then(/^the result should be \(ignoring element order for lists\):$/) do |table|
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).not_to be_nil
  expect(@query_result).to eq_in_order(GQLiteTest.parse_results_table table)
end

Then(/^the result should be, in order \(ignoring element order for lists\):$/) do |table|
  pending if @ignored_scenario
  expect(@exception).to be_nil
  expect(@query_result).not_to be_nil
  expect(@query_result).to eq_in_order(GQLiteTest.parse_results_table table)
end

Then(/^a SyntaxError should be raised at compile time: VariableAlreadyBound$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  # TODO: should not match for RunTime
  expect(@exception.message).to match(/^(RunTime)|(CompileTime): (VariableAlreadyBound: Variable '.*' is already bound( at \(\d+, \d+\))?\.)|(UnexpectedSyntax: Expected token .* got .* at \(\d+, \d+\)\.)$/)
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
  expect(@exception.message).to match(/^CompileTime: NoSingleRelationshipType:.*$/)
end

Then(/^a SyntaxError should be raised at compile time: RequiresDirectedRelationship$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: RequiresDirectedRelationship: edges need to be directed in this context: '.*'.$/)
end

Then(/^a SyntaxError should be raised at compile time: CreatingVarLength$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: Expected token .* got .* at \(\d+, \d+\)\.$/)
end

Then(/^a SyntaxError should be raised at compile time: InvalidParameterUse$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: ParseError: .*$/)
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
  expect(@exception.message).to match(/^CompileTime: ColumnNameConflict: Column '.*' is duplicated\.$/)
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

Then(/^a SyntaxError should be raised at compile time: InvalidAggregation$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: InvalidAggregation: aggregation is not accepted in this expression.$/)
end

Then(/^a ConstraintValidationFailed should be raised at runtime: DeleteConnectedNode$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: DeleteConnectedNode: .*\.$/)
end

Then(/^a TypeError should be raised at runtime: InvalidPropertyType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: .* \(\d+, \d+\)\.$/)
end

Then(/^an ArgumentError should be raised at runtime: InvalidArgumentType$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: InvalidArgumentType: .*\.$/)
end

Then(/^a ArgumentError should be raised at runtime: NegativeIntegerArgument$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: NegativeIntegerArgument: .*\.$/)
end

Then(/^a SyntaxError should be raised at compile time: AmbiguousAggregationExpression$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^CompileTime: UnexpectedSyntax: .* \(\d+, \d+\)\.$/)
end

Then(/^a TypeError should be raised at runtime: InvalidArgumentValue$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: InvalidArgumentValue: .*\.$/)
end

Then(/^an Error should be raised at any time: \*$/) do
  pending if @ignored_scenario
  expect(@exception).not_to be_nil
end

Then(/^an Error should be raised at run time: DuplicateGraphName$/) do
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: DuplicateGraphName: .*\.$/)
end

Then(/^an Error should be raised at run time: InexistingGraph$/) do
  expect(@exception).not_to be_nil
  expect(@exception.message).to match(/^RunTime: InexistingGraph: .*\.$/)
end
