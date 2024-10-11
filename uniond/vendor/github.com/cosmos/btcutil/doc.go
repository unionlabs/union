// Copyright (c) 2013-2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

/*
Package btcutil provides bitcoin-specific convenience functions and types.
Block Overview
A Block defines a bitcoin block that provides easier and more efficient
manipulation of raw wire protocol blocks.  It also memoizes hashes for the
block and its transactions on their first access so subsequent accesses don't
have to repeat the relatively expensive hashing operations.
Tx Overview
A Tx defines a bitcoin transaction that provides more efficient manipulation of
raw wire protocol transactions.  It memoizes the hash for the transaction on its
first access so subsequent accesses don't have to repeat the relatively
expensive hashing operations.
Address Overview
The Address interface provides an abstraction for a Bitcoin address.  While the
most common type is a pay-to-pubkey-hash, Bitcoin already supports others and
may well support more in the future.  This package currently provides
implementations for the pay-to-pubkey, pay-to-pubkey-hash, and
pay-to-script-hash address types.
To decode/encode an address:

	// NOTE: The default network is only used for address types which do not
	// already contain that information.  At this time, that is only
	// pay-to-pubkey addresses.
	addrString := "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962" +
		"e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d57" +
		"8a4c702b6bf11d5f"
	defaultNet := &chaincfg.MainNetParams
	addr, err := btcutil.DecodeAddress(addrString, defaultNet)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println(addr.EncodeAddress())
*/
package btcutil
