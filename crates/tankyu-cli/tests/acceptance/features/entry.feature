Feature: Entry management
  As a researcher
  I want to list, inspect, and update collected entries
  So that I can review what has been gathered and focus on what needs attention

  # --- List ---

  @wip
  Scenario: List entries when none exist
    When I run "entry list"
    Then the command exits successfully
    And stdout contains "No entries yet"

  Scenario: List entries shows all entries
    Given entries exist in the research graph
    When I run "entry list"
    Then the command exits successfully
    And stdout contains the entry titles

  Scenario: Filter entries by source
    Given a source "tokio-rs-tokio" exists with entries
    When I run "entry list --source tokio-rs-tokio"
    Then the command exits successfully
    And all listed entries belong to source "tokio-rs-tokio"

  # --- Unclassified (graph traversal via negation) ---

  Scenario: List unclassified entries excludes tagged entries
    Given an entry "alpha-post" is tagged with topic "rust"
    And an entry "beta-post" has no topic tags
    When I run "entry list --unclassified"
    Then the command exits successfully
    And stdout contains "beta-post"
    And stdout does not contain "alpha-post"

  Scenario: All entries unclassified when none are tagged
    Given entries exist in the research graph
    And no entries are tagged with any topic
    When I run "entry list --unclassified"
    Then the command exits successfully
    And all entries are listed

  # --- Inspect ---

  Scenario: Inspect an entry shows details and relationships
    Given an entry "alpha-post" exists with source "tokio-rs-tokio"
    And entry "alpha-post" is tagged with topic "rust"
    When I run "entry inspect alpha-post"
    Then the command exits successfully
    And stdout contains "alpha-post"
    And stdout contains "tokio-rs-tokio"
    And stdout contains "rust"

  @wip
  Scenario: Inspect a non-existent entry fails
    When I run "entry inspect does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  # --- Update ---

  Scenario: Update entry state
    Given an entry "alpha-post" exists
    When I run "entry update alpha-post --state read"
    Then the command exits successfully
    And stdout contains "Updated"
    And stdout contains "read"

  Scenario: Update entry signal
    Given an entry "alpha-post" exists
    When I run "entry update alpha-post --signal high"
    Then the command exits successfully
    And stdout contains "high"

  Scenario: Update without flags fails
    Given an entry "alpha-post" exists
    When I run "entry update alpha-post"
    Then the command exits with failure
    And stderr contains "at least one"

  @wip
  Scenario: Update a non-existent entry fails
    When I run "entry update does-not-exist --state read"
    Then the command exits with failure
    And stderr contains "not found"
