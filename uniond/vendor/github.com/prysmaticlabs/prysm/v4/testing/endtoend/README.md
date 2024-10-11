# End-to-end Testing Package

This is the main project folder of the end-to-end testing suite for Prysm. This performs a full end-to-end test for Prysm, including spinning up an ETH1 dev chain, sending deposits to the deposit contract, and making sure the beacon node and its validators are running and performing properly for a few epochs.
It also performs a test on a syncing node, and supports feature flags to allow easy E2E testing of experimental features. 

## How it works

Please see our docs page, https://docs.prylabs.network/docs/devtools/end-to-end, to read more about the feature.
