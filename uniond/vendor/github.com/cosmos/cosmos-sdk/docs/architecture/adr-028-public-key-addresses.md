# ADR 028: Public Key Addresses

## Changelog

* 2020/08/18: Initial version
* 2021/01/15: Analysis and algorithm update

## Status

Proposed

## Abstract

This ADR defines an address format for all addressable Cosmos SDK accounts. That includes: new public key algorithms, multisig public keys, and module accounts.

## Context

Issue [\#3685](https://github.com/cosmos/cosmos-sdk/issues/3685) identified that public key
address spaces are currently overlapping. We confirmed that it significantly decreases security of Cosmos SDK.

### Problem

An attacker can control an input for an address generation function. This leads to a birthday attack, which significantly decreases the security space.
To overcome this, we need to separate the inputs for different kind of account types:
a security break of one account type shouldn't impact the security of other account types.

### Initial proposals

One initial proposal was extending the address length and
adding prefixes for different types of addresses.

@ethanfrey explained an alternate approach originally used in https://github.com/iov-one/weave:

> I spent quite a bit of time thinking about this issue while building weave... The other cosmos Sdk.
> Basically I define a condition to be a type and format as human readable string with some binary data appended. This condition is hashed into an Address (again at 20 bytes). The use of this prefix makes it impossible to find a preimage for a given address with a different condition (eg ed25519 vs secp256k1).
> This is explained in depth here https://weave.readthedocs.io/en/latest/design/permissions.html
> And the code is here, look mainly at the top where we process conditions. https://github.com/iov-one/weave/blob/master/conditions.go

And explained how this approach should be sufficiently collision resistant:

> Yeah, AFAIK, 20 bytes should be collision resistance when the preimages are unique and not malleable. A space of 2^160 would expect some collision to be likely around 2^80 elements (birthday paradox). And if you want to find a collision for some existing element in the database, it is still 2^160. 2^80 only is if all these elements are written to state.
> The good example you brought up was eg. a public key bytes being a valid public key on two algorithms supported by the codec. Meaning if either was broken, you would break accounts even if they were secured with the safer variant. This is only as the issue when no differentiating type info is present in the preimage (before hashing into an address).
> I would like to hear an argument if the 20 bytes space is an actual issue for security, as I would be happy to increase my address sizes in weave. I just figured cosmos and ethereum and bitcoin all use 20 bytes, it should be good enough. And the arguments above which made me feel it was secure. But I have not done a deeper analysis.

This led to the first proposal (which we proved to be not good enough):
we concatenate a key type with a public key, hash it and take the first 20 bytes of that hash, summarized as `sha256(keyTypePrefix || keybytes)[:20]`.

### Review and Discussions

In [\#5694](https://github.com/cosmos/cosmos-sdk/issues/5694) we discussed various solutions.
We agreed that 20 bytes it's not future proof, and extending the address length is the only way to allow addresses of different types, various signature types, etc.
This disqualifies the initial proposal.

In the issue we discussed various modifications:

* Choice of the hash function.
* Move the prefix out of the hash function: `keyTypePrefix + sha256(keybytes)[:20]` [post-hash-prefix-proposal].
* Use double hashing: `sha256(keyTypePrefix + sha256(keybytes)[:20])`.
* Increase to keybytes hash slice from 20 byte to 32 or 40 bytes. We concluded that 32 bytes, produced by a good hash functions is future secure.

### Requirements

* Support currently used tools - we don't want to break an ecosystem, or add a long adaptation period. Ref: https://github.com/cosmos/cosmos-sdk/issues/8041
* Try to keep the address length small - addresses are widely used in state, both as part of a key and object value.

### Scope

This ADR only defines a process for the generation of address bytes. For end-user interactions with addresses (through the API, or CLI, etc.), we still use bech32 to format these addresses as strings. This ADR doesn't change that.
Using Bech32 for string encoding gives us support for checksum error codes and handling of user typos.

## Decision

We define the following account types, for which we define the address function:

1. simple accounts: represented by a regular public key (ie: secp256k1, sr25519)
2. naive multisig: accounts composed by other addressable objects (ie: naive multisig)
3. composed accounts with a native address key (ie: bls, group module accounts)
4. module accounts: basically any accounts which cannot sign transactions and which are managed internally by modules

### Legacy Public Key Addresses Don't Change

Currently (Jan 2021), the only officially supported Cosmos SDK user accounts are `secp256k1` basic accounts and legacy amino multisig.
They are used in existing Cosmos SDK zones. They use the following address formats:

* secp256k1: `ripemd160(sha256(pk_bytes))[:20]`
* legacy amino multisig: `sha256(aminoCdc.Marshal(pk))[:20]`

We don't want to change existing addresses. So the addresses for these two key types will remain the same.

The current multisig public keys use amino serialization to generate the address. We will retain
those public keys and their address formatting, and call them "legacy amino" multisig public keys
in protobuf. We will also create multisig public keys without amino addresses to be described below.

### Hash Function Choice

As in other parts of the Cosmos SDK, we will use `sha256`.

### Basic Address

We start with defining a base algorithm for generating addresses which we will call `Hash`. Notably, it's used for accounts represented by a single key pair. For each public key schema we have to have an associated `typ` string, explained in the next section. `hash` is the cryptographic hash function defined in the previous section.

```go
const A_LEN = 32

func Hash(typ string, key []byte) []byte {
    return hash(hash(typ) + key)[:A_LEN]
}
```

The `+` is bytes concatenation, which doesn't use any separator.

This algorithm is the outcome of a consultation session with a professional cryptographer.
Motivation: this algorithm keeps the address relatively small (length of the `typ` doesn't impact the length of the final address)
and it's more secure than [post-hash-prefix-proposal] (which uses the first 20 bytes of a pubkey hash, significantly reducing the address space).
Moreover the cryptographer motivated the choice of adding `typ` in the hash to protect against a switch table attack.

`address.Hash` is a low level function to generate _base_ addresses for new key types. Example:

* BLS: `address.Hash("bls", pubkey)`

### Composed Addresses

For simple composed accounts (like a new naive multisig) we generalize the `address.Hash`. The address is constructed by recursively creating addresses for the sub accounts, sorting the addresses and composing them into a single address. It ensures that the ordering of keys doesn't impact the resulting address.

```go
// We don't need a PubKey interface - we need anything which is addressable.
type Addressable interface {
    Address() []byte
}

func Composed(typ string, subaccounts []Addressable) []byte {
    addresses = map(subaccounts, \a -> LengthPrefix(a.Address()))
    addresses = sort(addresses)
    return address.Hash(typ, addresses[0] + ... + addresses[n])
}
```

The `typ` parameter should be a schema descriptor, containing all significant attributes with deterministic serialization (eg: utf8 string).
`LengthPrefix` is a function which prepends 1 byte to the address. The value of that byte is the length of the address bits before prepending. The address must be at most 255 bits long.
We are using `LengthPrefix` to eliminate conflicts - it assures, that for 2 lists of addresses: `as = {a1, a2, ..., an}` and `bs = {b1, b2, ..., bm}` such that every `bi` and `ai` is at most 255 long, `concatenate(map(as, (a) => LengthPrefix(a))) = map(bs, (b) => LengthPrefix(b))` if `as = bs`.

Implementation Tip: account implementations should cache addresses.

#### Multisig Addresses

For a new multisig public keys, we define the `typ` parameter not based on any encoding scheme (amino or protobuf). This avoids issues with non-determinism in the encoding scheme.

Example:

```protobuf
package cosmos.crypto.multisig;

message PubKey {
  uint32 threshold = 1;
  repeated google.protobuf.Any pubkeys = 2;
}
```

```go
func (multisig PubKey) Address() {
	// first gather all nested pub keys
	var keys []address.Addressable  // cryptotypes.PubKey implements Addressable
	for _, _key := range multisig.Pubkeys {
		keys = append(keys, key.GetCachedValue().(cryptotypes.PubKey))
	}

	// form the type from the message name (cosmos.crypto.multisig.PubKey) and the threshold joined together
	prefix := fmt.Sprintf("%s/%d", proto.MessageName(multisig), multisig.Threshold)

	// use the Composed function defined above
	return address.Composed(prefix, keys)
}
```


### Derived Addresses

We must be able to cryptographically derive one address from another one. The derivation process must guarantee hash properties, hence we use the already defined `Hash` function:

```go
func Derive(address, derivationKey []byte) []byte {
	return Hash(addres, derivationKey)
}
```

### Module Account Addresses

A module account will have `"module"` type. Module accounts can have sub accounts. The submodule account will be created based on module name, and sequence of derivation keys. Typically, the first derivation key should be a class of the derived accounts. The derivation process has a defined order: module name, submodule key, subsubmodule key... An example module account is created using:

```go
address.Module(moduleName, key)
```

An example sub-module account is created using:

```go
groupPolicyAddresses := []byte{1}
address.Module(moduleName, groupPolicyAddresses, policyID)
```

The `address.Module` function is using `address.Hash` with `"module"` as the type argument, and byte representation of the module name concatenated with submodule key. The two last component must be uniquely separated to avoid potential clashes (example: modulename="ab" & submodulekey="bc" will have the same derivation key as modulename="a" & submodulekey="bbc").
We use a null byte (`'\x00'`) to separate module name from the submodule key. This works, because null byte is not a part of a valid module name. Finally, the sub-submodule accounts are created by applying the `Derive` function recursively.
We could use `Derive` function also in the first step (rather than concatenating module name with zero byte and the submodule key). We decided to do concatenation to avoid one level of derivation and speed up computation.

For backward compatibility with the existing `authtypes.NewModuleAddress`, we add a special case in `Module` function: when no derivation key is provided, we fallback to the "legacy" implementation. 

```go
func Module(moduleName string, derivationKeys ...[]byte) []byte{
	if len(derivationKeys) == 0 {
		return authtypes.NewModuleAddress(modulenName)  // legacy case
	}
	submoduleAddress := Hash("module", []byte(moduleName) + 0 + key)
	return fold((a, k) => Derive(a, k), subsubKeys, submoduleAddress)
}
```

**Example 1**  A lending BTC pool address would be:

```go
btcPool := address.Module("lending", btc.Address()})
```

If we want to create an address for a module account depending on more than one key, we can concatenate them:

```go
btcAtomAMM := address.Module("amm", btc.Address() + atom.Address()})
```

**Example 2**  a smart-contract address could be constructed by:

```go
smartContractAddr = Module("mySmartContractVM", smartContractsNamespace, smartContractKey})

// which equals to:
smartContractAddr = Derived(
    Module("mySmartContractVM", smartContractsNamespace), 
    []{smartContractKey})
```

### Schema Types

A `typ` parameter used in `Hash` function SHOULD be unique for each account type.
Since all Cosmos SDK account types are serialized in the state, we propose to use the protobuf message name string.

Example: all public key types have a unique protobuf message type similar to:

```protobuf
package cosmos.crypto.sr25519;

message PubKey {
	bytes key = 1;
}
```

All protobuf messages have unique fully qualified names, in this example `cosmos.crypto.sr25519.PubKey`.
These names are derived directly from .proto files in a standardized way and used
in other places such as the type URL in `Any`s. We can easily obtain the name using
`proto.MessageName(msg)`.

## Consequences

### Backwards Compatibility

This ADR is compatible with what was committed and directly supported in the Cosmos SDK repository.

### Positive

* a simple algorithm for generating addresses for new public keys, complex accounts and modules
* the algorithm generalizes _native composed keys_
* increased security and collision resistance of addresses
* the approach is extensible for future use-cases - one can use other address types, as long as they don't conflict with the address length specified here (20 or 32 bytes).
* support new account types.

### Negative

* addresses do not communicate key type, a prefixed approach would have done this
* addresses are 60% longer and will consume more storage space
* requires a refactor of KVStore store keys to handle variable length addresses

### Neutral

* protobuf message names are used as key type prefixes

## Further Discussions

Some accounts can have a fixed name or may be constructed in other way (eg: modules). We were discussing an idea of an account with a predefined name (eg: `me.regen`), which could be used by institutions.
Without going into details, these kinds of addresses are compatible with the hash based addresses described here as long as they don't have the same length.
More specifically, any special account address must not have a length equal to 20 or 32 bytes.

## Appendix: Consulting session

End of Dec 2020 we had a session with [Alan Szepieniec](https://scholar.google.be/citations?user=4LyZn8oAAAAJ&hl=en) to consult the approach presented above.

Alan general observations:

* we don’t need 2-preimage resistance
* we need 32bytes address space for collision resistance
* when an attacker can control an input for object with an address then we have a problem with birthday attack
* there is an issue with smart-contracts for hashing
* sha2 mining can be use to breaking address pre-image

Hashing algorithm

* any attack breaking blake3 will break blake2
* Alan is pretty confident about the current security analysis of the blake hash algorithm. It was a finalist, and the author is well known in security analysis.

Algorithm:

* Alan recommends to hash the prefix: `address(pub_key) = hash(hash(key_type) + pub_key)[:32]`, main benefits:
    * we are free to user arbitrary long prefix names
    * we still don’t risk collisions
    * switch tables
* discussion about penalization -> about adding prefix post hash
* Aaron asked about post hash prefixes (`address(pub_key) = key_type + hash(pub_key)`) and differences. Alan noted that this approach has longer address space and it’s stronger.

Algorithm for complex / composed keys:

* merging tree like addresses with same algorithm are fine

Module addresses: Should module addresses have different size to differentiate it?

* we will need to set a pre-image prefix for module addresse to keept them in 32-byte space: `hash(hash('module') + module_key)`
* Aaron observation: we already need to deal with variable length (to not break secp256k1 keys).

Discssion about arithmetic hash function for ZKP

* Posseidon / Rescue
* Problem: much bigger risk because we don’t know much techniques and history of crypto-analysis of arithmetic constructions. It’s still a new ground and area of active research.

Post quantum signature size

* Alan suggestion: Falcon: speed / size ration - very good.
* Aaron - should we think about it?
  Alan: based on early extrapolation this thing will get able to break EC cryptography in 2050 . But that’s a lot of uncertainty. But there is magic happening with recurions / linking / simulation and that can speedup the progress.

Other ideas

* Let’s say we use same key and two different address algorithms for 2 different use cases. Is it still safe to use it? Alan: if we want to hide the public key (which is not our use case), then it’s less secure but there are fixes.

### References

* [Notes](https://hackmd.io/_NGWI4xZSbKzj1BkCqyZMw)
