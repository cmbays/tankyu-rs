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

  Scenario: Update entry state
    Given the data directory contains 3 entries with mixed state
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa --state read"
    Then the command exits successfully
    And stdout contains "Updated entry"
    And stdout contains "read"

  Scenario: Update entry signal
    Given the data directory contains 3 entries with mixed state
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa --signal high"
    Then the command exits successfully
    And stdout contains "high"

  Scenario: Update non-existent entry fails
    When I run "entry update 00000000-0000-0000-0000-000000000000 --state read"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: Update without flags fails
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    Then the command exits with failure
    And stderr contains "at least one"

  Scenario: List unclassified entries excludes classified entries
    Given the data directory contains 3 entries with mixed state
    And entry "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" is classified under topic "11111111-1111-1111-1111-111111111111"
    When I run "entry list --unclassified"
    Then the command exits successfully
    And stdout does not contain "Alpha entry"

  Scenario: List unclassified returns all entries when none classified
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --unclassified"
    Then the command exits successfully
    And stdout contains "Alpha entry"
    And stdout contains "Beta entry"
