// Package hd provides support for hierarchical deterministic wallets generation and derivation.
//
// The user must understand the overall concept of the BIP 32 and the BIP 44 specs:
//
//	https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
//	https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
//
// In combination with the bip39 package in go-crypto this package provides the functionality for
// deriving keys using a BIP 44 HD path, or, more general, by passing a BIP 32 path.
//
// In particular, this package (together with bip39) provides all necessary functionality to derive
// keys from mnemonics generated during the cosmos fundraiser.
package hd
