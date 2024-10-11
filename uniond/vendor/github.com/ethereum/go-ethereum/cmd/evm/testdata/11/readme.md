## Test missing basefee

In this test, the `currentBaseFee` is missing from the env portion. 
On a live blockchain, the basefee is present in the header, and verified as part of header validation. 

In `evm t8n`, we don't have blocks, so it needs to be added in the `env`instead. 

When it's missing, an error is expected. 

```
dir=./testdata/11 && ./evm t8n --state.fork=London --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --output.alloc=stdout --output.result=stdout 2>&1>/dev/null
ERROR(3): EIP-1559 config but missing 'currentBaseFee' in env section
```