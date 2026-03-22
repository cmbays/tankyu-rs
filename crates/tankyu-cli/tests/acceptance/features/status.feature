Feature: Research graph status
  As a researcher
  I want to see a dashboard of my research graph
  So that I know the size and shape of my knowledge base

  Scenario: Status on a fresh graph shows zero counts
    When I run "status"
    Then the command exits successfully
    And stdout contains "0 topics"
    And stdout contains "0 sources"
    And stdout contains "0 entries"

  @wip
  Scenario: Status reflects created data
    Given a topic exists with name "rust"
    And a source exists linked to topic "rust" with URL "https://github.com/tokio-rs/tokio"
    When I run "status"
    Then the command exits successfully
    And stdout contains "1 topic"
    And stdout contains "1 source"

  Scenario: Database auto-initializes on first command
    When I run "status"
    Then the command exits successfully
    And the research graph database exists
