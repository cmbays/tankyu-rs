Feature: System diagnostics
  As a researcher
  I want to verify my research graph is healthy
  So that I can catch configuration or database issues early

  Scenario: Doctor reports healthy system
    When I run "doctor"
    Then the command exits successfully
    And stdout contains "Database: OK"
    And stdout contains "Config: OK"

  Scenario: Doctor reports when database directory is missing
    Given no research graph database exists
    When I run "doctor"
    Then the command exits with failure
    And stdout contains "Database: not initialized"

  Scenario: Doctor reports config issues
    Given the configuration file is missing
    When I run "doctor"
    Then the command exits with failure
    And stdout contains "Config: not found"
