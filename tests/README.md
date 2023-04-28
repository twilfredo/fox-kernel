# Integration Tests

Purpose of these integration tests are to test various aspects of the kernel. Both the default/custom test frameworks will pickup and exec any tests in this directory.

Note that, all integration tests are their own executable (separated from `main.rs`). As such, each test must have it's own entry point function.

## Test Overview

### basic_boot.rs

Run a basic boot-up test.