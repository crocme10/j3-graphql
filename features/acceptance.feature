Feature: Non-Regression

  Scenario Outline: adding a new document
    Given there are no owners
    When I add a new document with the title <title>
    Then I find the document with a title <title> in the list of documents

    Examples:
      | title  |
      | bob    |
