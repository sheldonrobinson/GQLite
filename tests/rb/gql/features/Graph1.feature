Feature: Create Use
  Scenario: [1] Create Graph and Node, but not Use
    Given an empty graph
    And having executed:
      """
      CREATE GRAPH test1
      CREATE (:A)
      """
    When executing query:
      """
      MATCH (n)
      RETURN n
      """
    Then the result should be, in any order:
      | n |
    And no side effects
  Scenario: [2] Create Graph and Node, and Use
    Given an empty graph
    And having executed:
      """
      CREATE GRAPH test1
      CREATE (:A)
      """
    When executing query:
      """
      USE test1
      MATCH (n)
      RETURN n
      """
    Then the result should be, in any order:
      | n    |
      | (:A) |
    And no side effects
  Scenario: [3] Cannot Create Graph with existing name
    Given an empty graph
    When executing query:
      """
      CREATE GRAPH test1
      CREATE GRAPH test1
      CREATE (:A)
      """
    Then an Error should be raised at run time: DuplicatedGraph
  Scenario: [4] Cannot Use inexisting graph
    Given an empty graph
    When executing query:
      """
      USE test1
      """
    Then an Error should be raised at run time: UnknownGraph
