# ADR 034: PrivValidator file structure

## Changelog

03-11-2018: Initial Draft

## Context

For now, the PrivValidator file `priv_validator.json` contains mutable and immutable parts. 
Even in an insecure mode which does not encrypt private key on disk, it is reasonable to separate 
the mutable part and immutable part.

References:
[#1181](https://github.com/tendermint/tendermint/issues/1181)
[#2657](https://github.com/tendermint/tendermint/issues/2657)
[#2313](https://github.com/tendermint/tendermint/issues/2313)

## Proposed Solution

We can split mutable and immutable parts with two structs:
```go
// FilePVKey stores the immutable part of PrivValidator
type FilePVKey struct {
	Address types.Address  `json:"address"`
	PubKey  crypto.PubKey  `json:"pub_key"`
	PrivKey crypto.PrivKey `json:"priv_key"`

	filePath string
}

// FilePVState stores the mutable part of PrivValidator
type FilePVLastSignState struct {
	Height    int64        `json:"height"`
	Round     int          `json:"round"`
	Step      int8         `json:"step"`
	Signature []byte       `json:"signature,omitempty"`
	SignBytes cmn.HexBytes `json:"signbytes,omitempty"`

	filePath string
	mtx      sync.Mutex
}
```

Then we can combine `FilePVKey` with `FilePVLastSignState` and will get the original `FilePV`.

```go
type FilePV struct {
	Key           FilePVKey
	LastSignState FilePVLastSignState
}
```

As discussed, `FilePV` should be located in `config`, and `FilePVLastSignState` should be stored in `data`. The 
store path of each file should be specified in `config.yml`.

What we need to do next is changing the methods of `FilePV`.

## Status

Implemented

## Consequences

### Positive

- separate the mutable and immutable of PrivValidator

### Negative

- need to add more config for file path

### Neutral
