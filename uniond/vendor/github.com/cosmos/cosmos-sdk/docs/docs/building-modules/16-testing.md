---
sidebar_position: 1
---

# Testing

The Cosmos SDK contains different types of [tests](https://martinfowler.com/articles/practical-test-pyramid.html).
These tests have different goals and are used at different stages of the development cycle.
We advice, as a general rule, to use tests at all stages of the development cycle.
It is adviced, as a chain developer, to test your application and modules in a similar way than the SDK.

The rationale behind testing can be found in [ADR-59](https://docs.cosmos.network/main/architecture/adr-059-test-scopes.html).

## Unit Tests

Unit tests are the lowest test category of the [test pyramid](https://martinfowler.com/articles/practical-test-pyramid.html).
All packages and modules should have unit test coverage. Modules should have their dependencies mocked: this means mocking keepers.

The SDK uses `mockgen` to generate mocks for keepers:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/scripts/mockgen.sh#L3-L6
```

You can read more about mockgen [here](https://github.com/golang/mock).

### Example

As an example, we will walkthrough the [keeper tests](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/keeper/keeper_test.go) of the `x/gov` module.

The `x/gov` module has a `Keeper` type requires a few external dependencies (ie. imports outside `x/gov` to work properly).

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/keeper/keeper.go#L61-L65
```

In order to only test `x/gov`, we mock the [expected keepers](https://docs.cosmos.network/v0.46/building-modules/keeper.html#type-definition) and instantiate the `Keeper` with the mocked dependencies. Note that we may need to configure the mocked dependencies to return the expected values:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/keeper/common_test.go#L67-L81
```

This allows us to test the `x/gov` module without having to import other modules.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/keeper/keeper_test.go#L3-L35
```

We can test then create unit tests using the newly created `Keeper` instance.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/keeper/keeper_test.go#L73-L91
```

## Integration Tests

Integration tests are at the second level of the [test pyramid](https://martinfowler.com/articles/practical-test-pyramid.html).
In the SDK, we locate our integration tests under [`/tests/integrations`](https://github.com/cosmos/cosmos-sdk/tree/main/tests/integration).

The goal of these integration tests is to test a component with a minimal application (i.e. not `simapp`). The minimal application is defined with the help of [`depinject`](../tooling/02-depinject.md) – the SDK dependency injection framework, and includes all necessary modules to test the component. With the helps of the SDK testing package, we can easily create a minimal application and start the application with a set of genesis transactions: <https://github.com/cosmos/cosmos-sdk/blob/main/testutil/sims/app_helpers.go>.

### Example

Here, we will walkthrough the integration tests of the `x/distribution` module. The `x/distribution` module has, in addition to keeper unit tests, integration tests that test the `x/distribution` module with a minimal application. This is expected as you may want to test the `x/distribution` module with actual application logic, instead of only mocked dependencies.

For creating a minimal application, we use [`simtestutil.Setup`](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/testutil/sims/app_helpers.go#L95-L99) and an [`AppConfig`](../tooling/02-depinject.md) of the `x/distribution` minimal dependencies.

For instance, the `AppConfig` of `x/distribution` is defined as:

* https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/distribution/testutil/app_config.go

This is a stripped down version of the `simapp` `AppConfig`:

* https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app_config.go

:::note
You can as well use the `AppConfig` `configurator` for creating an `AppConfig` [inline](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/slashing/app_test.go#L54-L62). There no difference between those two ways, use whichever you prefer.
:::

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/tests/integration/distribution/keeper/keeper_test.go#L28-L33
```

Now the types are injected and we can use them for our tests:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/tests/integration/distribution/keeper/keeper_test.go#L21-L53
```

## Deterministic and Regression tests	

Tests are written for queries in the Cosmos SDK which have `module_query_safe` Protobuf annotation.

Each query is tested using 2 methods:

* Use property-based testing with the [`rapid`](https://pkg.go.dev/pgregory.net/rapid@v0.5.3) library. The property that is tested is that the query response and gas consumption are the same upon 1000 query calls.
* Regression tests are written with hardcoded responses and gas, and verify they don't change upon 1000 calls and between SDK patch versions.

Here's an example of regression tests:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/tests/integration/bank/keeper/deterministic_test.go#L102-L115
```

## Simulations

Simulations uses as well a minimal application, built with [`depinject`](../tooling/02-depinject.md):

Following is an example for `x/gov/` simulations:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/simulation/operations_test.go#L292-L310
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/simulation/operations_test.go#L69-L111
```

## End-to-end Tests

End-to-end tests are at the top of the [test pyramid](https://martinfowler.com/articles/practical-test-pyramid.html).
They must test the whole application flow, from the user perspective (for instance, CLI tests). They are located under [`/tests/e2e`](https://github.com/cosmos/cosmos-sdk/tree/main/tests/e2e).

For that, the SDK is using `simapp` but you should use your own application (`appd`).
Here are some examples:

* SDK E2E tests: <https://github.com/cosmos/cosmos-sdk/tree/main/tests/e2e>.
* Cosmos Hub E2E tests: <https://github.com/cosmos/gaia/tree/main/tests/e2e>.
* Osmosis E2E tests: <https://github.com/osmosis-labs/osmosis/tree/main/tests/e2e>.

:::note warning
The SDK is in the process of creating its E2E tests, as defined in [ADR-59](https://docs.cosmos.network/main/architecture/adr-059-test-scopes.html). This page will eventually be updated with better examples.
:::

## Summary

| Scope       | App Fixture | Mocks? |
| ----------- | ----------- | ------ |
| Unit        | None        | Yes    |
| Integration | `depinject` | Some   |
| Simulation  | `depinject` | No     |
| E2E         | `appd`      | No     |
