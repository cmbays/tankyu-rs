Feature: Topic management
  As a researcher
  I want to create, list, and inspect research topics
  So that I can organize my research focus areas

  # --- Create ---

  Scenario: Create a topic
    When I run "topic create rust-async"
    Then the command exits successfully
    And stdout contains "Created topic: rust-async"

  @wip
  Scenario: Create a topic generates a slug
    When I run "topic create rust-async"
    Then the command exits successfully
    And stdout contains "slug: rust-async"

  Scenario: Create a duplicate topic fails
    Given a topic exists with name "existing-topic"
    When I run "topic create existing-topic"
    Then the command exits with failure
    And stderr contains "already exists"

  # --- List ---

  @wip
  Scenario: List topics when none exist
    When I run "topic list"
    Then the command exits successfully
    And stdout contains "No topics yet"

  Scenario: List topics shows all topics
    Given a topic exists with name "rust"
    And a topic exists with name "wasm"
    When I run "topic list"
    Then the command exits successfully
    And stdout contains "rust"
    And stdout contains "wasm"

  # --- Inspect ---

  Scenario: Inspect a topic shows details and relationships
    Given a topic exists with name "rust"
    And a source exists linked to topic "rust" with URL "https://github.com/tokio-rs/tokio"
    When I run "topic inspect rust"
    Then the command exits successfully
    And stdout contains "rust"
    And stdout contains "tokio-rs/tokio"

  @wip
  Scenario: Inspect a non-existent topic fails
    When I run "topic inspect does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"
