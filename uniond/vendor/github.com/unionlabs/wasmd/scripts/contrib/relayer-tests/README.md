# Relayer tests

These scripts helps to test go-relayer with two local wasmd chains. \
Make sure you run below scripts under `wasmd/scripts/contrib/relayer-tests` directory.

- `./init_two_chainz_relayer.sh` will spin two chains and runs
- `./one_chain.sh` will spin a single chain. This script used by the one above
- `./test_ibc_transfer.sh` will setup a path between chains and send tokens between chains.

## Thank you
The setup scripts here are taken from [cosmos/relayer](https://github.com/cosmos/relayer)
Thank your relayer team for these scripts.


