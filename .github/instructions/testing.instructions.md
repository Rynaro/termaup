---
applyTo: "**/tests/**,**/*test*"
---

# Testing Instructions

## Rules
- Use `#[tokio::test]` for all async tests
- Use `wiremock::MockServer` for HTTP mocking — configure realistic response bodies
- Never make real HTTP calls to the ClickUp API in tests
- Use `insta` for snapshot testing of formatted output
- Test fixtures (JSON files) go in a `fixtures/` directory relative to the test file
- Test function names must be descriptive: `test_<what>_<condition>_<expected>`
- Each test should test ONE thing
- Use `assert_eq!` with descriptive messages: `assert_eq!(result.name, "Expected", "task name should match")`
- `.unwrap()` is acceptable in tests for brevity
