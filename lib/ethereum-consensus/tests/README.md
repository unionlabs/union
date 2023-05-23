# Tests from Ethereum Consensus Spec Tests

The consensus spec tests repo is [here](https://github.com/ethereum/consensus-spec-tests).

In order to run tests against the consensus spec test repo, use the following commands:

```
just download-integration-tests
just run-integration-tests
```

The tests are currently behind a feature `spec-tests` which is enabled by default in `Cargo.toml`. If you don't want to run these tests, you can run `cargo` with the default features disabled.
