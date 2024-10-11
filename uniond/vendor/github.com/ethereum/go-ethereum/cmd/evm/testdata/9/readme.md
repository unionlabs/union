## EIP-1559 testing

This test contains testcases for EIP-1559, which uses a new transaction type and has a new block parameter.

### Prestate

The alloc portion contains one contract (`0x000000000000000000000000000000000000aaaa`), containing the 
following code: `0x58585454`: `PC; PC; SLOAD; SLOAD`.

Essentially, this contract does `SLOAD(0)` and `SLOAD(1)`.

The alloc also contains some funds on `0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b`. 

## Transactions

There are two transactions, each invokes the contract above. 

1. EIP-1559 ACL-transaction, which contains the `0x0` slot for `0xaaaa`
2. Legacy transaction

## Execution 

Running it yields: 
```
$ dir=./testdata/9 && ./evm t8n --state.fork=London --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --trace 2>/dev/null  && cat trace-*  | grep SLOAD
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x47c86","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":2,"op":84,"gas":"0x48c28","gasCost":"0x834","memSize":0,"stack":["0x0","0x1"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":3,"op":84,"gas":"0x483f4","gasCost":"0x64","memSize":0,"stack":["0x0","0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x49cf6","gasCost":"0x834","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x494be","gasCost":"0x834","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":2,"op":84,"gas":"0x49cf4","gasCost":"0x834","memSize":0,"stack":["0x0","0x1"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":3,"op":84,"gas":"0x494c0","gasCost":"0x834","memSize":0,"stack":["0x0","0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":1,"op":84,"gas":"0x484be","gasCost":"0x64","memSize":0,"stack":["0x0"],"depth":1,"refund":0,"opName":"SLOAD"}
{"pc":4,"op":84,"gas":"0x48456","gasCost":"0x64","memSize":0,"stack":["0x3"],"depth":1,"refund":0,"opName":"SLOAD"}
```

We can also get the post-alloc:
```
$ dir=./testdata/9 && ./evm t8n --state.fork=London --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --output.alloc=stdout 2>/dev/null
{
  "alloc": {
    "0x000000000000000000000000000000000000aaaa": {
      "code": "0x58585454",
      "balance": "0x3",
      "nonce": "0x1"
    },
    "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba": {
      "balance": "0x5bb10ddef6e0"
    },
    "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
      "balance": "0xff745ee8832120",
      "nonce": "0x2"
    }
  }
}
```

If we try to execute it on older rules: 
```
dir=./testdata/9 && ./evm t8n --state.fork=Berlin --input.alloc=$dir/alloc.json --input.txs=$dir/txs.json --input.env=$dir/env.json --output.alloc=stdout
ERROR(10): Failed signing transactions: ERROR(10): Tx 0: failed to sign tx: transaction type not supported
```

It fails, due to the `evm t8n` cannot sign them in with the given signer. We can bypass that, however, 
by feeding it presigned transactions, located in `txs_signed.json`. 

```
dir=./testdata/9 && ./evm t8n --state.fork=Berlin --input.alloc=$dir/alloc.json --input.txs=$dir/txs_signed.json --input.env=$dir/env.json 
WARN [03-09|11:06:22.065] rejected tx                              index=0 hash=334e09..f8dce5 error="transaction type not supported"
INFO [03-09|11:06:22.066] rejected tx                              index=1 hash=a9c6c6..fa4036 from=0xa94f5374Fce5edBC8E2a8697C15331677e6EbF0B error="nonce too high: address 0xa94f5374Fce5edBC8E2a8697C15331677e6EbF0B, tx: 1 state: 0"
INFO [03-09|11:06:22.066] Trie dumping started                     root=6eebe9..a0fda5
INFO [03-09|11:06:22.066] Trie dumping complete                    accounts=2 elapsed="55.844µs"
INFO [03-09|11:06:22.066] Wrote file                               file=alloc.json
INFO [03-09|11:06:22.066] Wrote file                               file=result.json
```

Number `0` is not applicable, and therefore number `1` has wrong nonce, and both are rejected.

