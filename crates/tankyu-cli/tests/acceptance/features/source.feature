Feature: Source management
  As a researcher
  I want to add, inspect, list, and remove sources
  So that I can manage the information sources I track

  # --- Add (ADR-1: idempotent) ---

  Scenario: Add a new source
    When I run "source add https://github.com/tokio-rs/tokio"
    Then the command exits successfully
    And stdout contains "tokio-rs-tokio"
    And stdout contains "github-repo"

  Scenario: Add a source linked to a topic creates a monitors edge
    Given a topic exists with name "async-rust"
    When I run "source add https://github.com/tokio-rs/tokio --topic async-rust"
    Then the command exits successfully
    And stdout contains "tokio-rs-tokio"
    And stdout contains "Linked to topic: async-rust"

  Scenario: Add the same source again is idempotent
    Given a source exists with URL "https://github.com/tokio-rs/tokio"
    When I run "source add https://github.com/tokio-rs/tokio"
    Then the command exits successfully
    And stdout contains "already exists"

  Scenario: Add a source to a non-existent topic fails
    When I run "source add https://example.com --topic no-such-topic"
    Then the command exits with failure
    And stderr contains "topic not found"

  Scenario: Source type is detected from URL
    When I run "source add https://github.com/rust-lang/rust/issues"
    Then the command exits successfully
    And stdout contains "github-issues"

  # --- List ---

  Scenario: List sources when none exist
    When I run "source list"
    Then the command exits successfully
    And stdout contains "No sources yet"

  Scenario: List sources shows all sources
    Given a source exists with URL "https://github.com/tokio-rs/tokio"
    And a source exists with URL "https://blog.rust-lang.org"
    When I run "source list"
    Then the command exits successfully
    And stdout contains "tokio-rs-tokio"
    And stdout contains "blog-rust-lang-org"

  # --- Inspect ---

  Scenario: Inspect a source shows details and relationships
    Given a topic exists with name "rust"
    And a source exists linked to topic "rust" with URL "https://github.com/tokio-rs/tokio"
    When I run "source inspect tokio-rs-tokio"
    Then the command exits successfully
    And stdout contains "tokio-rs-tokio"
    And stdout contains "github-repo"
    And stdout contains "rust"

  Scenario: Inspect a non-existent source fails
    When I run "source inspect does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  # --- Remove (edges auto-cascade) ---

  Scenario: Remove a source
    Given a topic exists with name "rust"
    And a source exists linked to topic "rust" with URL "https://github.com/tokio-rs/tokio"
    When I run "source remove tokio-rs-tokio"
    Then the command exits successfully
    And stdout contains "Removed source: tokio-rs-tokio"

  Scenario: Removing a source also removes its edges
    Given a topic exists with name "rust"
    And a source exists linked to topic "rust" with URL "https://github.com/tokio-rs/tokio"
    When I run "source remove tokio-rs-tokio"
    And I run "topic inspect rust"
    Then stdout does not contain "tokio-rs-tokio"

  Scenario: Remove a non-existent source fails
    When I run "source remove does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"
