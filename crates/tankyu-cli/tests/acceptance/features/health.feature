Feature: Source health checking
  As a researcher
  I want to know which sources are stale, dormant, or empty
  So that I can maintain the quality of my research pipeline

  Scenario: All sources healthy
    Given a source exists that was checked recently with entries
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Never-checked source is flagged as stale
    Given a source exists that has never been checked
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout contains "never checked"

  Scenario: Source not checked within stale threshold
    Given a source exists last checked 10 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"

  Scenario: Source not checked within dormant threshold
    Given a source exists last checked 35 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "dormant"

  Scenario: Source checked at exactly stale threshold is not stale
    Given a source exists last checked 7 days ago
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Source checked at exactly dormant threshold is not dormant
    Given a source exists last checked 30 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout does not contain "dormant"

  Scenario: Source with no entries is flagged as empty
    Given a source exists that has no entries
    When I run "health"
    Then the command exits with failure
    And stdout contains "empty"

  Scenario: Pruned sources are ignored
    Given only a pruned source exists
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Thresholds come from configuration
    Given config has stale_days set to 3
    And a source exists last checked 4 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
