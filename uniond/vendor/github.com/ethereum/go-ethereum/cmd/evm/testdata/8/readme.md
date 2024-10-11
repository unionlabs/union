## EIP-2930 testing

This test contains testcases for EIP-2930, which uses transactions with access lists. 

### Prestate

The alloc portion contains one contract (`0x000000000000000000000000000000000000aaaa`), containing the 
following code: `0x5854505854`: `PC ;SLOAD; POP; PC; SLOAD`.

Essentially, this contract does `SLOAD(0)` and `SLOAD(3)`.

The alloc also contains some funds on `0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b`. 

## Transactions

There are three transactions, each invokes the contract above. 

1. ACL-transaction, which contains some non-used slots
2. Regular transaction
3. ACL-transaction, which contains the slots `1` and `3` in `0x000000000000000000000000000000000000aaaa`

## Execution 

Running it yields: 
```
dir=./testdata/8 && ./evm t8n --state.fork=Berlin --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --trace 2>/dev/null && cat trace-* | grep SLOAD
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x47c86","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x49cf6","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x494be","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x64","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x48456","gasCost":"0x64","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
```

Similarly, we can provide the input transactions via `stdin` instead of as file:

```
$ dir=./testdata/8 \
  && cat $dir/txs.json | jq "{txs: .}" \
  | ./evm t8n --state.fork=Berlin \
     --input.alloc=$dir/alloc.json \
     --input.txs=stdin \
     --input.env=$dir/env.json \
     --trace  \
     2>/dev/null \
  && cat trace-* | grep SLOAD
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x47c86","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x49cf6","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x494be","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x64","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x48456","gasCost":"0x64","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
```

If we try to execute it on older rules: 
```
$ dir=./testdata/8 && ./evm t8n --state.fork=Istanbul --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json 
ERROR(10): failed signing transactions: ERROR(10): tx 0: failed to sign tx: transaction type not supported
```
