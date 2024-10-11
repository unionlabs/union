---
sidebar_position: 1
---

# System Tests

System tests provide a framework to write and execute black box tests against a running chain. This adds another level
of confidence on top of unit, integration, and simulations tests, ensuring that business-critical scenarios
(like double signing prevention) or scenarios that can't be tested otherwise (like a chain upgrade) are covered.

## Vanilla Go for Flow Control

System tests are vanilla Go tests that interact with the compiled chain binary. The `test runner` component starts a
local testnet of 4 nodes (by default) and provides convenient helper methods for accessing the
`system under test (SUT)`.
A `CLI wrapper` makes it easy to access keys, submit transactions, or execute operations. Together, these components
enable the replication and validation of complex business scenarios.

Here's an example of a double signing test, where a new node is added with the same key as the first validator:
[double signing test example](https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/tests/systemtests/fraud_test.go)

The [getting started tutorial](https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/tests/systemtests/getting_started.md)
contains a step-by-step guide to building and running your first system test. It covers setting chain state via genesis
or
transactions and validation via transaction response or queries.

## Design Principles and Guidelines

System tests are slower compared to unit or integration tests as they interact with a running chain. Therefore, certain
principles can guide their usage:

- **Perspective:** Tests should mimic a human interacting with the chain from the outside. Initial states can be set via
  genesis or transactions to support a test scenario.
- **Roles:** The user can have multiple roles such as validator, delegator, granter, or group admin.
- **Focus:** Tests should concentrate on happy paths or business-critical workflows. Unit and integration tests are
  better suited for more fine-grained testing.
- **Workflows:** Test workflows and scenarios, not individual units. Given the high setup costs, it is reasonable to
  combine multiple steps and assertions in a single test method.
- **Genesis Mods:** Genesis modifications can incur additional time costs for resetting dirty states. Reuse existing
  accounts (node0..n) whenever possible.
- **Framework:** Continuously improve the framework for better readability and reusability.

## Errors and Debugging

All output is logged to `systemtests/testnet/node{0..n}.out`. Usually, `node0.out` is very noisy as it receives the CLI
connections. Prefer any other node's log to find stack traces or error messages.

Using system tests for state setup during debugging has become very handy:

- Start the test with one node only and verbose output:

  ```sh
  go test -v -tags=system_test ./ --run TestAccountCreation --verbose --nodes-count=1
  ```

- Copy the CLI command for the transaction and modify the test to stop before the command
- Start the node with `--home=<project-home>/tests/systemtests/testnet/node0/<binary-name>/` in debug mode
- Execute CLI command from shell and enter breakpoints
