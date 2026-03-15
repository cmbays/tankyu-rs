Feature: Source health checking
  As a researcher
  I want to know which sources are stale, dormant, or empty
  So that I can maintain the quality of my research pipeline

  Scenario: All sources healthy exits 0
    Given a source exists with name "fresh-source" checked 1 day ago with entries
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Never-checked source produces stale warning
    Given a source exists with name "never-checked" that has never been checked
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout contains "never checked"

  Scenario: Stale source produces warning and exits 1
    Given a source exists with name "stale-source" last checked 10 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout contains "stale-source"

  Scenario: Dormant source produces warning and exits 1
    Given a source exists with name "dormant-source" last checked 35 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "dormant"
    And stdout contains "dormant-source"

  Scenario: Empty source produces warning and exits 1
    Given a source exists with name "empty-source" that has no entries
    When I run "health"
    Then the command exits with failure
    And stdout contains "empty"

  Scenario: Pruned source is ignored
    Given a pruned source exists with name "pruned-source"
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Health report as JSON
    Given a source exists with name "fresh-source" checked 1 day ago with entries
    When I run "health --json"
    Then the command exits successfully
