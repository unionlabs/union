# Invalid rlp

This folder contains a sample of invalid RLP, and it's expected
that the t9n handles this properly: 

```
$ go run . t9n --input.txs=./testdata/18/invalid.rlp --state.fork=London 
ERROR(11): rlp: value size exceeds available input length
```