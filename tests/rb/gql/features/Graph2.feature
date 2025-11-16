Feature: Create Use
  Scenario: [1] Create Graph and drop it
    Given an empty graph
    When executing query:
      """
      CREATE GRAPH test1
      DROP GRAPH test1
      """
    Then the result should be empty
    And no side effects
  Scenario: [2] Create Graph and drop it and try to use it
    Given an empty graph
    And having executed:
      """
      CREATE GRAPH test1
      DROP GRAPH test1
      """
    When executing query:
      """
      USE test1
      """
    Then an Error should be raised at run time: UnknownGraph
  Scenario: [3] Dropping an unexisting graph should fail
    Given an empty graph
    When executing query:
      """
      DROP GRAPH test1
      """
    Then an Error should be raised at run time: UnknownGraph
  Scenario: [4] Dropping an unexisting graph should succeed
    Given an empty graph
    When executing query:
      """
      DROP GRAPH IF EXISTS test1
      """
    Then the result should be empty
