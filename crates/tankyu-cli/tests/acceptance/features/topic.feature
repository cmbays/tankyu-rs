Feature: Topic management
  As a researcher
  I want to create and list research topics
  So that I can organize my research focus areas

  Scenario: Create a topic
    When I run "topic create Rust-Async"
    Then the command exits successfully
    And stdout contains "Created topic: Rust-Async"

  Scenario: Create a topic with tags
    When I run "topic create Systems --tags rust,c,cpp"
    Then the command exits successfully
    And stdout contains "Tags: rust, c, cpp"

  Scenario: Create a duplicate topic fails
    Given a topic exists with name "Existing-Topic"
    When I run "topic create Existing-Topic"
    Then the command exits with failure
    And stderr contains "already exists"
