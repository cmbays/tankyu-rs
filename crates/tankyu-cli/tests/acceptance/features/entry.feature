Feature: Entry management
  As a researcher
  I want to list and inspect my collected entries
  So that I can review what has been gathered and focus on what needs attention

  Scenario: List entries in table format when entries exist
    Given the data directory contains 3 entries with mixed state
    When I run "entry list"
    Then the command exits successfully
    And stdout contains "new"

  Scenario: List all entries as JSON
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --json"
    Then the command exits successfully
    And stdout is a JSON array of length 3

  Scenario: Filter entries by state new
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --state new"
    Then the command exits successfully
    And stdout contains "Alpha entry"
    And stdout does not contain "Beta entry"

  Scenario: Invalid state flag is rejected
    When I run "entry list --state garbage"
    Then the command exits with failure
    And stderr contains "Invalid state"

  Scenario: topic and source flags are mutually exclusive
    When I run "entry list --topic foo --source bar"
    Then the command exits with failure
    And stderr contains "mutually exclusive"

  Scenario: No entries shows empty table
    When I run "entry list"
    Then the command exits successfully
    And stdout does not contain "feat:"

  Scenario: Inspect a non-existent entry fails
    When I run "entry inspect 00000000-0000-0000-0000-000000000000"
    Then the command exits with failure
    And stderr contains "not found"
