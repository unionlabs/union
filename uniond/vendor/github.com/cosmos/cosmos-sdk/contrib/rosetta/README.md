# rosetta

This directory contains the files required to run the rosetta CI. It builds `simapp` based on the current codebase.

## docker-compose.yaml

Builds:

* cosmos-sdk simapp node, with prefixed data directory, keys etc. This is required to test historical balances.
* faucet is required so we can test construction API, it was literally impossible to put there a deterministic address to request funds for
* rosetta is the rosetta node used by rosetta-cli to interact with the cosmos-sdk app
* test_rosetta runs the rosetta-cli test against construction API and data API

## configuration

Contains the required files to set up rosetta cli and make it work against its workflows

## Rosetta-ci

Contains the files for a deterministic network, with fixed keys and some actions on there, to test parsing of msgs and historical balances.  This image is used to run a simapp node and to run the rosetta server and the rosetta-cli.
Whenever [rosetta-cli](https://github.com/coinbase/rosetta-cli) releases a new version, rosetta-ci/Dockerfile should be updated to reflect the new version.

## Notes

* Keyring password is 12345678
* data.sh creates node data, it's required in case consensus breaking changes are made to quickly recreate replicable node data for rosetta
