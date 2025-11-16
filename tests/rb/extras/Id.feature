Feature: Create Use
  Scenario: [1] Create two nodes, match one by Id
    Given an empty graph
    When executing query:
      """
      CREATE (n {id: 1}), (m {id: 2})
      MATCH (p) WHERE id(p) = id(n)
      RETURN p.id AS pid
      """
    Then the result should be, in any order:
      | pid |
      | 1   |
    And the side effects should be:
      | +nodes      | 2 |
      | +properties | 2 |
