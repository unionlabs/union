## EIP 4788

This test contains testcases for EIP-4788. The 4788-contract is 
located at address `0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02`, and this test executes a simple transaction. It also
implicitly invokes the system tx, which sets calls the contract and sets the 
storage values

```
$ dir=./testdata/29/ && go run . t8n --state.fork=Cancun  --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --output.alloc=stdout
INFO [09-27|15:34:53.049] Trie dumping started                     root=19a4f8..01573c
INFO [09-27|15:34:53.049] Trie dumping complete                    accounts=2 elapsed="192.759µs"
INFO [09-27|15:34:53.050] Wrote file                               file=result.json
{
  "alloc": {
    "0x000f3df6d732807ef1319fb7b8bb8522d0beac02": {
      "code": "0x3373fffffffffffffffffffffffffffffffffffffffe14604457602036146024575f5ffd5b620180005f350680545f35146037575f5ffd5b6201800001545f5260205ff35b6201800042064281555f359062018000015500",
      "storage": {
        "0x000000000000000000000000000000000000000000000000000000000000079e": "0x000000000000000000000000000000000000000000000000000000000000079e",
        "0x000000000000000000000000000000000000000000000000000000000001879e": "0x0000beac00beac00beac00beac00beac00beac00beac00beac00beac00beac00"
      },
      "balance": "0x1"
    },
    "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
      "balance": "0x16345785d871db8",
      "nonce": "0x1"
    }
  }
}
```
