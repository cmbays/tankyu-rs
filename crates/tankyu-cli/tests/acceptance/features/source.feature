Feature: Source management
  As a researcher
  I want to add, inspect, and remove sources
  So that I can manage the information sources I track

  Scenario: Inspect an existing source
    Given a source exists with name "rust-lang/rust" and URL "https://github.com/rust-lang/rust"
    When I run "source inspect rust-lang/rust"
    Then the command exits successfully
    And stdout contains "rust-lang/rust"
    And stdout contains "github-repo"
    And stdout contains "active"

  Scenario: Inspect a non-existent source fails
    When I run "source inspect does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: Add a new GitHub repo source
    When I run "source add https://github.com/tokio-rs/tokio"
    Then the command exits successfully
    And stdout contains "tokio-rs/tokio"
    And stdout contains "github-repo"

  Scenario: Remove a source marks it as pruned
    Given a source exists with name "old-source" and URL "https://example.com/old"
    When I run "source remove old-source"
    Then the command exits successfully
    And stdout contains "marked as pruned"

  Scenario: Remove a non-existent source fails
    When I run "source remove does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: List sources shows empty hint when no sources exist
    When I run "source list"
    Then the command exits successfully
    And stdout contains "No sources yet"

  Scenario: Add a source linked to a topic creates monitors edge
    Given a topic exists with name "AsyncRust"
    When I run "source add https://github.com/tokio-rs/tokio --topic AsyncRust"
    Then the command exits successfully
    And stdout contains "tokio-rs/tokio"
    And stdout contains "Linked to topic: AsyncRust"
