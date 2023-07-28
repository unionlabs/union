bech32
==========

[![Build Status](http://img.shields.io/travis/cosmos/btcutil.svg)](https://travis-ci.org/cosmos/btcutil)
[![ISC License](http://img.shields.io/badge/license-ISC-blue.svg)](http://copyfree.org)
[![GoDoc](https://godoc.org/github.com/cosmos/btcutil/bech32?status.png)](http://godoc.org/github.com/cosmos/btcutil/bech32)

Package bech32 provides a Go implementation of the bech32 format specified in
[BIP 173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki).

Test vectors from BIP 173 are added to ensure compatibility with the BIP.

## Installation and Updating

```bash
$ go get -u github.com/cosmos/btcutil/bech32
```

## Examples

* [Bech32 decode Example](http://godoc.org/github.com/cosmos/btcutil/bech32#example-Bech32Decode)
  Demonstrates how to decode a bech32 encoded string.
* [Bech32 encode Example](http://godoc.org/github.com/cosmos/btcutil/bech32#example-BechEncode)
  Demonstrates how to encode data into a bech32 string.

## License

Package bech32 is licensed under the [copyfree](http://copyfree.org) ISC
License.
