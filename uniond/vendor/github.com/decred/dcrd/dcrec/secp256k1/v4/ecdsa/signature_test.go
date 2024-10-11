// Copyright (c) 2013-2016 The btcsuite developers
// Copyright (c) 2015-2023 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package ecdsa

import (
	"bytes"
	"encoding/hex"
	"errors"
	"math/rand"
	"testing"
	"time"

	"github.com/decred/dcrd/crypto/blake256"
	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// hexToBytes converts the passed hex string into bytes and will panic if there
// is an error.  This is only provided for the hard-coded constants so errors in
// the source code can be detected. It will only (and must only) be called with
// hard-coded values.
func hexToBytes(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	return b
}

// TestSignatureParsing ensures that signatures are properly parsed according
// to DER rules.  The error paths are tested as well.
func TestSignatureParsing(t *testing.T) {
	tests := []struct {
		name string
		sig  []byte
		err  error
	}{{
		// signature from Decred blockchain tx
		// 76634e947f49dfc6228c3e8a09cd3e9e15893439fc06df7df0fc6f08d659856c:0
		name: "valid signature 1",
		sig: hexToBytes("3045022100cd496f2ab4fe124f977ffe3caa09f7576d8a34156" +
			"b4e55d326b4dffc0399a094022013500a0510b5094bff220c74656879b8ca03" +
			"69d3da78004004c970790862fc03"),
		err: nil,
	}, {
		// signature from Decred blockchain tx
		// 76634e947f49dfc6228c3e8a09cd3e9e15893439fc06df7df0fc6f08d659856c:1
		name: "valid signature 2",
		sig: hexToBytes("3044022036334e598e51879d10bf9ce3171666bc2d1bbba6164" +
			"cf46dd1d882896ba35d5d022056c39af9ea265c1b6d7eab5bc977f06f81e35c" +
			"dcac16f3ec0fd218e30f2bad2a"),
		err: nil,
	}, {
		name: "empty",
		sig:  nil,
		err:  ErrSigTooShort,
	}, {
		name: "too short",
		sig:  hexToBytes("30050201000200"),
		err:  ErrSigTooShort,
	}, {
		name: "too long",
		sig: hexToBytes("3045022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074022030e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef8481352480101"),
		err: ErrSigTooLong,
	}, {
		name: "bad ASN.1 sequence id",
		sig: hexToBytes("3145022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074022030e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidSeqID,
	}, {
		name: "mismatched data length (short one byte)",
		sig: hexToBytes("3044022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074022030e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidDataLen,
	}, {
		name: "mismatched data length (long one byte)",
		sig: hexToBytes("3046022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074022030e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidDataLen,
	}, {
		name: "bad R ASN.1 int marker",
		sig: hexToBytes("304403204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d6" +
			"24c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56c" +
			"bbac4622082221a8768d1d09"),
		err: ErrSigInvalidRIntID,
	}, {
		name: "zero R length",
		sig: hexToBytes("30240200022030e09575e7a1541aa018876a4003cefe1b061a90" +
			"556b5140c63e0ef848135248"),
		err: ErrSigZeroRLen,
	}, {
		name: "negative R (too little padding)",
		sig: hexToBytes("30440220b2ec8d34d473c3aa2ab5eb7cc4a0783977e5db8c8daf" +
			"777e0b6d7bfa6b6623f302207df6f09af2c40460da2c2c5778f636d3b2e27e20" +
			"d10d90f5a5afb45231454700"),
		err: ErrSigNegativeR,
	}, {
		name: "too much R padding",
		sig: hexToBytes("304402200077f6e93de5ed43cf1dfddaa79fca4b766e1a8fc879" +
			"b0333d377f62538d7eb5022054fed940d227ed06d6ef08f320976503848ed1f5" +
			"2d0dd6d17f80c9c160b01d86"),
		err: ErrSigTooMuchRPadding,
	}, {
		name: "bad S ASN.1 int marker",
		sig: hexToBytes("3045022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074032030e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidSIntID,
	}, {
		name: "missing S ASN.1 int marker",
		sig: hexToBytes("3023022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074"),
		err: ErrSigMissingSTypeID,
	}, {
		name: "S length missing",
		sig: hexToBytes("3024022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef07402"),
		err: ErrSigMissingSLen,
	}, {
		name: "invalid S length (short one byte)",
		sig: hexToBytes("3045022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074021f30e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidSLen,
	}, {
		name: "invalid S length (long one byte)",
		sig: hexToBytes("3045022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef074022130e09575e7a1541aa018876a4003cefe1b061a" +
			"90556b5140c63e0ef848135248"),
		err: ErrSigInvalidSLen,
	}, {
		name: "zero S length",
		sig: hexToBytes("3025022100f5353150d31a63f4a0d06d1f5a01ac65f7267a719e" +
			"49f2a1ac584fd546bef0740200"),
		err: ErrSigZeroSLen,
	}, {
		name: "negative S (too little padding)",
		sig: hexToBytes("304402204fc10344934662ca0a93a84d14d650d8a21cf2ab91f6" +
			"08e8783d2999c955443202208441aacd6b17038ff3f6700b042934f9a6fea0ce" +
			"c2051b51dc709e52a5bb7d61"),
		err: ErrSigNegativeS,
	}, {
		name: "too much S padding",
		sig: hexToBytes("304402206ad2fdaf8caba0f2cb2484e61b81ced77474b4c2aa06" +
			"9c852df1351b3314fe20022000695ad175b09a4a41cd9433f6b2e8e83253d6a7" +
			"402096ba313a7be1f086dde5"),
		err: ErrSigTooMuchSPadding,
	}, {
		name: "R == 0",
		sig: hexToBytes("30250201000220181522ec8eca07de4860a4acdd12909d831cc5" +
			"6cbbac4622082221a8768d1d09"),
		err: ErrSigRIsZero,
	}, {
		name: "R == N",
		sig: hexToBytes("3045022100fffffffffffffffffffffffffffffffebaaedce6af" +
			"48a03bbfd25e8cd03641410220181522ec8eca07de4860a4acdd12909d831cc5" +
			"6cbbac4622082221a8768d1d09"),
		err: ErrSigRTooBig,
	}, {
		name: "R > N (>32 bytes)",
		sig: hexToBytes("3045022101cd496f2ab4fe124f977ffe3caa09f756283910fc1a" +
			"96f60ee6873e88d3cfe1d50220181522ec8eca07de4860a4acdd12909d831cc5" +
			"6cbbac4622082221a8768d1d09"),
		err: ErrSigRTooBig,
	}, {
		name: "R > N",
		sig: hexToBytes("3045022100fffffffffffffffffffffffffffffffebaaedce6af" +
			"48a03bbfd25e8cd03641420220181522ec8eca07de4860a4acdd12909d831cc5" +
			"6cbbac4622082221a8768d1d09"),
		err: ErrSigRTooBig,
	}, {
		name: "S == 0",
		sig: hexToBytes("302502204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d6" +
			"24c6c61548ab5fb8cd41020100"),
		err: ErrSigSIsZero,
	}, {
		name: "S == N",
		sig: hexToBytes("304502204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d6" +
			"24c6c61548ab5fb8cd41022100fffffffffffffffffffffffffffffffebaaedc" +
			"e6af48a03bbfd25e8cd0364141"),
		err: ErrSigSTooBig,
	}, {
		name: "S > N (>32 bytes)",
		sig: hexToBytes("304502204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d6" +
			"24c6c61548ab5fb8cd4102210113500a0510b5094bff220c74656879b784b246" +
			"ba89c0a07bc49bcf05d8993d44"),
		err: ErrSigSTooBig,
	}, {
		name: "S > N",
		sig: hexToBytes("304502204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d6" +
			"24c6c61548ab5fb8cd41022100fffffffffffffffffffffffffffffffebaaedc" +
			"e6af48a03bbfd25e8cd0364142"),
		err: ErrSigSTooBig,
	}}

	for _, test := range tests {
		_, err := ParseDERSignature(test.sig)
		if !errors.Is(err, test.err) {
			t.Errorf("%s mismatched err -- got %v, want %v", test.name, err,
				test.err)
			continue
		}
	}
}

// TestSignatureSerialize ensures that serializing signatures works as expected.
func TestSignatureSerialize(t *testing.T) {
	tests := []struct {
		name     string
		ecsig    *Signature
		expected []byte
	}{{
		// signature from bitcoin blockchain tx
		// 0437cd7f8525ceed2324359c2d0ba26006d92d85
		"valid 1 - r and s most significant bits are zero",
		&Signature{
			r: *hexToModNScalar("4e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd41"),
			s: *hexToModNScalar("181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d09"),
		},
		hexToBytes("304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d62" +
			"4c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc" +
			"56cbbac4622082221a8768d1d09"),
	}, {
		// signature from bitcoin blockchain tx
		// cb00f8a0573b18faa8c4f467b049f5d202bf1101d9ef2633bc611be70376a4b4
		"valid 2 - r most significant bit is one",
		&Signature{
			r: *hexToModNScalar("82235e21a2300022738dabb8e1bbd9d19cfb1e7ab8c30a23b0afbb8d178abcf3"),
			s: *hexToModNScalar("24bf68e256c534ddfaf966bf908deb944305596f7bdcc38d69acad7f9c868724"),
		},
		hexToBytes("304502210082235e21a2300022738dabb8e1bbd9d19cfb1e7ab8c" +
			"30a23b0afbb8d178abcf3022024bf68e256c534ddfaf966bf908deb94430" +
			"5596f7bdcc38d69acad7f9c868724"),
	}, {
		// signature from bitcoin blockchain tx
		// fda204502a3345e08afd6af27377c052e77f1fefeaeb31bdd45f1e1237ca5470
		//
		// Note that signatures with an S component that is > half the group
		// order are neither allowed nor produced in Decred, so this has been
		// modified to expect the equally valid low S signature variant.
		"valid 3 - s most significant bit is one",
		&Signature{
			r: *hexToModNScalar("1cadddc2838598fee7dc35a12b340c6bde8b389f7bfd19a1252a17c4b5ed2d71"),
			s: *hexToModNScalar("c1a251bbecb14b058a8bd77f65de87e51c47e95904f4c0e9d52eddc21c1415ac"),
		},
		hexToBytes("304402201cadddc2838598fee7dc35a12b340c6bde8b389f7bfd1" +
			"9a1252a17c4b5ed2d7102203e5dae44134eb4fa757428809a2178199e66f" +
			"38daa53df51eaa380cab4222b95"),
	}, {
		"zero signature",
		&Signature{
			r: *new(secp256k1.ModNScalar).SetInt(0),
			s: *new(secp256k1.ModNScalar).SetInt(0),
		},
		hexToBytes("3006020100020100"),
	}}

	for i, test := range tests {
		result := test.ecsig.Serialize()
		if !bytes.Equal(result, test.expected) {
			t.Errorf("Serialize #%d (%s) unexpected result:\n"+
				"got:  %x\nwant: %x", i, test.name, result,
				test.expected)
		}
	}
}

// signTest describes tests for producing and verifying ECDSA signatures for a
// selected set of private keys, messages, and nonces that have been verified
// independently with the Sage computer algebra system.  It is defined
// separately since it is intended for use in both normal and compact signature
// tests.
type signTest struct {
	name     string // test description
	key      string // hex encoded private key
	msg      string // hex encoded message to sign before hashing
	hash     string // hex encoded hash of the message to sign
	nonce    string // hex encoded nonce to use in the signature calculation
	rfc6979  bool   // whether or not the nonce is an RFC6979 nonce
	wantSigR string // hex encoded expected signature R
	wantSigS string // hex encoded expected signature S
	wantCode byte   // expected public key recovery code
}

// signTests returns several tests for ECDSA signatures that use a selected set
// of private keys, messages, and nonces that have been verified independently
// with the Sage computer algebra system.  It is defined here versus inside a
// specific test function scope so it can be shared for both normal and compact
// signature tests.
func signTests(t *testing.T) []signTest {
	t.Helper()

	tests := []signTest{{
		name:     "key 0x1, blake256(0x01020304), rfc6979 nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000001",
		msg:      "01020304",
		hash:     "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		nonce:    "4154324ecd4158938f1df8b5b659aeb639c7fbc36005934096e514af7d64bcc2",
		rfc6979:  true,
		wantSigR: "c6c4137b0e5fbfc88ae3f293d7e80c8566c43ae20340075d44f75b009c943d09",
		wantSigS: "00ba213513572e35943d5acdd17215561b03f11663192a7252196cc8b2a99560",
		wantCode: 0,
	}, {
		name:     "key 0x1, blake256(0x01020304), random nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000001",
		msg:      "01020304",
		hash:     "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		nonce:    "a6df66500afeb7711d4c8e2220960855d940a5ed57260d2c98fbf6066cca283e",
		rfc6979:  false,
		wantSigR: "b073759a96a835b09b79e7b93c37fdbe48fb82b000c4a0e1404ba5d1fbc15d0a",
		wantSigS: "7e34928a3e3832ec21e7711644d9388f7deb6340ead661d7056b0665974b87f3",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "key 0x2, blake256(0x01020304), rfc6979 nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000002",
		msg:      "01020304",
		hash:     "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		nonce:    "55f96f24cf7531f527edfe3b9222eca12d575367c32a7f593a828dc3651acf49",
		rfc6979:  true,
		wantSigR: "e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59",
		wantSigS: "44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "key 0x2, blake256(0x01020304), random nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000002",
		msg:      "01020304",
		hash:     "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		nonce:    "679a6d36e7fe6c02d7668af86d78186e8f9ccc04371ac1c8c37939d1f5cae07a",
		rfc6979:  false,
		wantSigR: "4a090d82f48ca12d9e7aa24b5dcc187ee0db2920496f671d63e86036aaa7997e",
		wantSigS: "261ffe8ba45007fc5fbbba6b4c6ed41beafb48b09fa8af1d6a3fbc6ccefbad",
		wantCode: 0,
	}, {
		name:     "key 0x1, blake256(0x0102030405), rfc6979 nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000001",
		msg:      "0102030405",
		hash:     "dc063eba3c8d52a159e725c1a161506f6cb6b53478ad5ef3f08d534efa871d9f",
		nonce:    "aa87a543c68f2568bb107c9946afa5233bf94fb6a7a063544505282621021629",
		rfc6979:  true,
		wantSigR: "dda8308cdbda2edf51ccf598b42b42b19597e102eb2ed4a04a16dd57084d3b40",
		wantSigS: "0b6d67bab4929624e28f690407a15efc551354544fdc179970ff401eec2e5dc9",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "key 0x1, blake256(0x0102030405), random nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000001",
		msg:      "0102030405",
		hash:     "dc063eba3c8d52a159e725c1a161506f6cb6b53478ad5ef3f08d534efa871d9f",
		nonce:    "65f880c892fdb6e7f74f76b18c7c942cfd037ef9cf97c39c36e08bbc36b41616",
		rfc6979:  false,
		wantSigR: "72e5666f4e9d1099447b825cf737ee32112f17a67e2ca7017ae098da31dfbb8b",
		wantSigS: "1a7326da661a62f66358dcf53300afdc8e8407939dae1192b5b0899b0254311b",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "key 0x2, blake256(0x0102030405), rfc6979 nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000002",
		msg:      "0102030405",
		hash:     "dc063eba3c8d52a159e725c1a161506f6cb6b53478ad5ef3f08d534efa871d9f",
		nonce:    "a13d652abd54b6e862548e5d12716df14dc192d93f3fa13536fdf4e56c54f233",
		rfc6979:  true,
		wantSigR: "122663fd29e41a132d3c8329cf05d61ebcca9351074cc277dcd868faba58d87d",
		wantSigS: "353a44f2d949c04981e4e4d9c1f93a9e0644e63a5eaa188288c5ad68fd288d40",
		wantCode: 0,
	}, {
		name:     "key 0x2, blake256(0x0102030405), random nonce",
		key:      "0000000000000000000000000000000000000000000000000000000000000002",
		msg:      "0102030405",
		hash:     "dc063eba3c8d52a159e725c1a161506f6cb6b53478ad5ef3f08d534efa871d9f",
		nonce:    "026ece4cfb704733dd5eef7898e44c33bd5a0d749eb043f48705e40fa9e9afa0",
		rfc6979:  false,
		wantSigR: "3c4c5a2f217ea758113fd4e89eb756314dfad101a300f48e5bd764d3b6e0f8bf",
		wantSigS: "6513e82442f133cb892514926ed9158328ead488ff1b027a31827603a65009df",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "random key 1, blake256(0x01), rfc6979 nonce",
		key:      "a1becef2069444a9dc6331c3247e113c3ee142edda683db8643f9cb0af7cbe33",
		msg:      "01",
		hash:     "4a6c419a1e25c85327115c4ace586decddfe2990ed8f3d4d801871158338501d",
		nonce:    "edb3a01063a0c6ccfc0d77295077cbd322cf364bfa64b7eeea3b20305135d444",
		rfc6979:  true,
		wantSigR: "ef392791d87afca8256c4c9c68d981248ee34a09069f50fa8dfc19ae34cd92ce",
		wantSigS: "0a2b9cb69fd794f7f204c272293b8585a294916a21a11fd94ec04acae2dc6d21",
		wantCode: 0,
	}, {
		name:     "random key 2, blake256(0x02), rfc6979 nonce",
		key:      "59930b76d4b15767ec0e8c8e5812aa2e57db30c6af7963e2a6295ba02af5416b",
		msg:      "02",
		hash:     "49af37ab5270015fe25276ea5a3bb159d852943df23919522a202205fb7d175c",
		nonce:    "af2a59085976494567ef0fc2ecede587b2d1d8e9898cc46e72d7f3e33156e057",
		rfc6979:  true,
		wantSigR: "886c9cccb356b3e1deafef2c276a4f8717ab73c1244c3f673cfbff5897de0e06",
		wantSigS: "609394185495f978ae84b69be90c69947e5dd8dcb4726da604fcbd139d81fc55",
		wantCode: 0,
	}, {
		name:     "random key 3, blake256(0x03), rfc6979 nonce",
		key:      "c5b205c36bb7497d242e96ec19a2a4f086d8daa919135cf490d2b7c0230f0e91",
		msg:      "03",
		hash:     "b706d561742ad3671703c247eb927ee8a386369c79644131cdeb2c5c26bf6c5d",
		nonce:    "82d82b696a386d6d7a111c4cb943bfd39de8e5f6195e7eed9d3edb40fe1419fa",
		rfc6979:  true,
		wantSigR: "6589d5950cec1fe2e7e20593b5ffa3556de20c176720a1796aa77a0cec1ec5a7",
		wantSigS: "2a26deba3241de852e786f5b4e2b98d3efb958d91fe9773b331dbcca9e8be800",
		wantCode: 0,
	}, {
		name:     "random key 4, blake256(0x04), rfc6979 nonce",
		key:      "65b46d4eb001c649a86309286aaf94b18386effe62c2e1586d9b1898ccf0099b",
		msg:      "04",
		hash:     "4c6eb9e38415034f4c93d3304d10bef38bf0ad420eefd0f72f940f11c5857786",
		nonce:    "7afd696a9e770961d2b2eaec77ab7c22c734886fa57bc4a50a9f1946168cd06f",
		rfc6979:  true,
		wantSigR: "81db1d6dca08819ad936d3284a359091e57c036648d477b96af9d8326965a7d1",
		wantSigS: "1bdf719c4be69351ba7617a187ac246912101aea4b5a7d6dfc234478622b43c6",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "random key 5, blake256(0x05), rfc6979 nonce",
		key:      "915cb9ba4675de06a182088b182abcf79fa8ac989328212c6b866fa3ec2338f9",
		msg:      "05",
		hash:     "bdd15db13448905791a70b68137445e607cca06cc71c7a58b9b2e84a06c54d08",
		nonce:    "2a6ae70ea5cf1b932331901d640ece54551f5f33bf9484d5f95c676b5612b527",
		rfc6979:  true,
		wantSigR: "47fd51aecbc743477cb59aa29d18d11d75fb206ae1cdd044216e4f294e33d5b6",
		wantSigS: "3d50edc03066584d50b8d19d681865a23960b37502ede5bf452bdca56744334a",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "random key 6, blake256(0x06), rfc6979 nonce",
		key:      "93e9d81d818f08ba1f850c6dfb82256b035b42f7d43c1fe090804fb009aca441",
		msg:      "06",
		hash:     "19b7506ad9c189a9f8b063d2aee15953d335f5c88480f8515d7d848e7771c4ae",
		nonce:    "0b847a0ae0cbe84dfca66621f04f04b0f2ec190dce10d43ba8c3915c0fcd90ed",
		rfc6979:  true,
		wantSigR: "c99800bc7ac7ea11afe5d7a264f4c26edd63ae9c7ecd6d0d19992980bcda1d34",
		wantSigS: "2844d4c9020ddf9e96b86c1a04788e0f371bd562291fd17ee017db46259d04fb",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "random key 7, blake256(0x07), rfc6979 nonce",
		key:      "c249bbd5f533672b7dcd514eb1256854783531c2b85fe60bf4ce6ea1f26afc2b",
		msg:      "07",
		hash:     "53d661e71e47a0a7e416591200175122d83f8af31be6a70af7417ad6f54d0038",
		nonce:    "0f8e20694fe766d7b79e5ac141e3542f2f3c3d2cc6d0f60e0ec263a46dbe6d49",
		rfc6979:  true,
		wantSigR: "7a57a5222fb7d615eaa0041193f682262cebfa9b448f9c519d3644d0a3348521",
		wantSigS: "574923b7b5aec66b62f1589002db29342c9f5ed56d5e80f5361c0307ff1561fa",
		wantCode: 0,
	}, {
		name:     "random key 8, blake256(0x08), rfc6979 nonce",
		key:      "ec0be92fcec66cf1f97b5c39f83dfd4ddcad0dad468d3685b5eec556c6290bcc",
		msg:      "08",
		hash:     "9bff7982eab6f7883322edf7bdc86a23c87ca1c07906fbb1584f57b197dc6253",
		nonce:    "ab7df49257d18f5f1b730cc7448f46bd82eb43e6e220f521fa7d23802310e24d",
		rfc6979:  true,
		wantSigR: "64f90b09c8b1763a3eeefd156e5d312f80a98c24017811c0163b1c0b01323668",
		wantSigS: "7d7bf4ff295ecfc9578eadc8378b0eea0c0362ad083b0fd1c9b3c06f4537f6ff",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}, {
		name:     "random key 9, blake256(0x09), rfc6979 nonce",
		key:      "6847b071a7cba6a85099b26a9c3e57a964e4990620e1e1c346fecc4472c4d834",
		msg:      "09",
		hash:     "4c2231813064f8500edae05b40195416bd543fd3e76c16d6efb10c816d92e8b6",
		nonce:    "48ea6c907e1cda596048d812439ccf416eece9a7de400c8a0e40bd48eb7e613a",
		rfc6979:  true,
		wantSigR: "81fc600775d3cdcaa14f8629537299b8226a0c8bfce9320ce64a8d14e3f95bae",
		wantSigS: "3607997d36b48bce957ae9b3d450e0969f6269554312a82bf9499efc8280ea6d",
		wantCode: 0,
	}, {
		name:     "random key 10, blake256(0x0a), rfc6979 nonce",
		key:      "b7548540f52fe20c161a0d623097f827608c56023f50442cc00cc50ad674f6b5",
		msg:      "0a",
		hash:     "e81db4f0d76e02805155441f50c861a8f86374f3ae34c7a3ff4111d3a634ecb1",
		nonce:    "95c07e315cd5457e84270ca01019563c8eeaffb18ab4f23e88a44a0ff01c5f6f",
		rfc6979:  true,
		wantSigR: "0d4cbf2da84f7448b083fce9b9c4e1834b5e2e98defcec7ec87e87c739f5fe78",
		wantSigS: "0997db60683e12b4494702347fc7ae7f599e5a95c629c146e0fc615a1a2acac5",
		wantCode: pubKeyRecoveryCodeOddnessBit,
	}}

	// Ensure the test data is sane by comparing the provided hashed message and
	// nonce, in the case RFC6979 was used, to their calculated values.  These
	// values could just be calculated instead of specified in the test data,
	// but it's nice to have all of the calculated values available in the test
	// data for cross implementation testing and verification.
	for _, test := range tests {
		msg := hexToBytes(test.msg)
		hash := hexToBytes(test.hash)

		calcHash := blake256.Sum256(msg)
		if !bytes.Equal(calcHash[:], hash) {
			t.Errorf("%s: mismatched test hash -- expected: %x, given: %x",
				test.name, calcHash[:], hash)
			continue
		}
		if test.rfc6979 {
			privKeyBytes := hexToBytes(test.key)
			nonceBytes := hexToBytes(test.nonce)
			calcNonce := secp256k1.NonceRFC6979(privKeyBytes, hash, nil, nil, 0)
			calcNonceBytes := calcNonce.Bytes()
			if !bytes.Equal(calcNonceBytes[:], nonceBytes) {
				t.Errorf("%s: mismatched test nonce -- expected: %x, given: %x",
					test.name, calcNonceBytes, nonceBytes)
				continue
			}
		}
	}

	return tests
}

// TestSignAndVerify ensures the ECDSA signing function produces the expected
// signatures for a selected set of private keys, messages, and nonces that have
// been verified independently with the Sage computer algebra system.  It also
// ensures verifying the signature works as expected.
func TestSignAndVerify(t *testing.T) {
	t.Parallel()

	tests := signTests(t)
	for _, test := range tests {
		privKey := secp256k1.NewPrivateKey(hexToModNScalar(test.key))
		hash := hexToBytes(test.hash)
		nonce := hexToModNScalar(test.nonce)
		wantSigR := hexToModNScalar(test.wantSigR)
		wantSigS := hexToModNScalar(test.wantSigS)
		wantSig := NewSignature(wantSigR, wantSigS).Serialize()

		// Sign the hash of the message with the given private key and nonce.
		gotSig, recoveryCode, success := sign(&privKey.Key, nonce, hash)
		if !success {
			t.Errorf("%s: unexpected error when signing", test.name)
			continue
		}

		// Ensure the generated signature is the expected value.
		gotSigBytes := gotSig.Serialize()
		if !bytes.Equal(gotSigBytes, wantSig) {
			t.Errorf("%s: unexpected signature -- got %x, want %x", test.name,
				gotSigBytes, wantSig)
			continue
		}

		// Ensure the generated public key recovery code is the expected value.
		if recoveryCode != test.wantCode {
			t.Errorf("%s: unexpected recovery code -- got %x, want %x",
				test.name, recoveryCode, test.wantCode)
			continue
		}

		// Ensure the R method returns the expected value.
		gotSigR := gotSig.R()
		if !gotSigR.Equals(wantSigR) {
			t.Errorf("%s: unexpected R component -- got %064x, want %064x",
				test.name, gotSigR.Bytes(), wantSigR.Bytes())
		}

		// Ensure the S method returns the expected value.
		gotSigS := gotSig.S()
		if !gotSigS.Equals(wantSigS) {
			t.Errorf("%s: unexpected S component -- got %064x, want %064x",
				test.name, gotSigS.Bytes(), wantSigS.Bytes())
		}

		// Ensure the produced signature verifies.
		pubKey := privKey.PubKey()
		if !gotSig.Verify(hash, pubKey) {
			t.Errorf("%s: signature failed to verify", test.name)
			continue
		}

		// Ensure the signature generated by the exported method is the expected
		// value as well in the case RFC6979 was used.
		if test.rfc6979 {
			gotSig = Sign(privKey, hash)
			gotSigBytes := gotSig.Serialize()
			if !bytes.Equal(gotSigBytes, wantSig) {
				t.Errorf("%s: unexpected signature -- got %x, want %x",
					test.name, gotSigBytes, wantSig)
				continue
			}
		}
	}
}

// TestSignAndVerifyRandom ensures ECDSA signing and verification work as
// expected for randomly-generated private keys and messages.  It also ensures
// invalid signatures are not improperly verified by mutating the valid
// signature and changing the message the signature covers.
func TestSignAndVerifyRandom(t *testing.T) {
	t.Parallel()

	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := rand.New(rand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	for i := 0; i < 100; i++ {
		// Generate a random private key.
		var buf [32]byte
		if _, err := rng.Read(buf[:]); err != nil {
			t.Fatalf("failed to read random private key: %v", err)
		}
		var privKeyScalar secp256k1.ModNScalar
		privKeyScalar.SetBytes(&buf)
		privKey := secp256k1.NewPrivateKey(&privKeyScalar)

		// Generate a random hash to sign.
		var hash [32]byte
		if _, err := rng.Read(hash[:]); err != nil {
			t.Fatalf("failed to read random hash: %v", err)
		}

		// Sign the hash with the private key and then ensure the produced
		// signature is valid for the hash and public key associated with the
		// private key.
		sig := Sign(privKey, hash[:])
		pubKey := privKey.PubKey()
		if !sig.Verify(hash[:], pubKey) {
			t.Fatalf("failed to verify signature\nsig: %x\nhash: %x\n"+
				"private key: %x\npublic key: %x", sig.Serialize(), hash,
				privKey.Serialize(), pubKey.SerializeCompressed())
		}

		// Change a random bit in the signature and ensure the bad signature
		// fails to verify the original message.
		badSig := *sig
		randByte := rng.Intn(32)
		randBit := rng.Intn(7)
		if randComponent := rng.Intn(2); randComponent == 0 {
			badSigBytes := badSig.r.Bytes()
			badSigBytes[randByte] ^= 1 << randBit
			badSig.r.SetBytes(&badSigBytes)
		} else {
			badSigBytes := badSig.s.Bytes()
			badSigBytes[randByte] ^= 1 << randBit
			badSig.s.SetBytes(&badSigBytes)
		}
		if badSig.Verify(hash[:], pubKey) {
			t.Fatalf("verified bad signature\nsig: %x\nhash: %x\n"+
				"private key: %x\npublic key: %x", badSig.Serialize(), hash,
				privKey.Serialize(), pubKey.SerializeCompressed())
		}

		// Change a random bit in the hash that was originally signed and ensure
		// the original good signature fails to verify the new bad message.
		badHash := make([]byte, len(hash))
		copy(badHash, hash[:])
		randByte = rng.Intn(len(badHash))
		randBit = rng.Intn(7)
		badHash[randByte] ^= 1 << randBit
		if sig.Verify(badHash, pubKey) {
			t.Fatalf("verified signature for bad hash\nsig: %x\nhash: %x\n"+
				"pubkey: %x", sig.Serialize(), badHash,
				pubKey.SerializeCompressed())
		}
	}
}

// TestSignFailures ensures the internal ECDSA signing function returns an
// unsuccessful result when particular combinations of values are unable to
// produce a valid signature.
func TestSignFailures(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name  string // test description
		key   string // hex encoded private key
		hash  string // hex encoded hash of the message to sign
		nonce string // hex encoded nonce to use in the signature calculation
	}{{
		name:  "zero R is invalid (forced by using zero nonce)",
		key:   "0000000000000000000000000000000000000000000000000000000000000001",
		hash:  "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		nonce: "0000000000000000000000000000000000000000000000000000000000000000",
	}, {
		name:  "zero S is invalid (forced by key/hash/nonce choice)",
		key:   "0000000000000000000000000000000000000000000000000000000000000001",
		hash:  "393bec84f1a04037751c0d6c2817f37953eaa204ac0898de7adb038c33a20438",
		nonce: "4154324ecd4158938f1df8b5b659aeb639c7fbc36005934096e514af7d64bcc2",
	}}

	for _, test := range tests {
		privKey := hexToModNScalar(test.key)
		hash := hexToBytes(test.hash)
		nonce := hexToModNScalar(test.nonce)

		// Ensure the signing is NOT successful.
		sig, _, success := sign(privKey, nonce, hash)
		if success {
			t.Errorf("%s: unexpected success -- got sig %x", test.name,
				sig.Serialize())
			continue
		}
	}
}

// TestVerifyFailures ensures the ECDSA verification function returns an
// unsuccessful result for edge conditions.
func TestVerifyFailures(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name string // test description
		key  string // hex encoded private key
		hash string // hex encoded hash of the message to sign
		r, s string // hex encoded r and s components of signature to verify
	}{{
		name: "signature R is 0",
		key:  "0000000000000000000000000000000000000000000000000000000000000001",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		r:    "0000000000000000000000000000000000000000000000000000000000000000",
		s:    "00ba213513572e35943d5acdd17215561b03f11663192a7252196cc8b2a99560",
	}, {
		name: "signature S is 0",
		key:  "0000000000000000000000000000000000000000000000000000000000000001",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		r:    "c6c4137b0e5fbfc88ae3f293d7e80c8566c43ae20340075d44f75b009c943d09",
		s:    "0000000000000000000000000000000000000000000000000000000000000000",
	}, {
		name: "u1G + u2Q is the point at infinity",
		key:  "0000000000000000000000000000000000000000000000000000000000000001",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		r:    "3cfe45621a29fac355260a14b9adc0fe43ac2f13e918fc9ddfa117e964b61a8a",
		s:    "00ba213513572e35943d5acdd17215561b03f11663192a7252196cc8b2a99560",
	}, {
		name: "signature R < P-N, but invalid",
		key:  "0000000000000000000000000000000000000000000000000000000000000001",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		r:    "000000000000000000000000000000014551231950b75fc4402da1722fc9baed",
		s:    "00ba213513572e35943d5acdd17215561b03f11663192a7252196cc8b2a99560",
	}}

	for _, test := range tests {
		privKey := hexToModNScalar(test.key)
		hash := hexToBytes(test.hash)
		r := hexToModNScalar(test.r)
		s := hexToModNScalar(test.s)
		sig := NewSignature(r, s)

		// Ensure the verification is NOT successful.
		pubKey := secp256k1.NewPrivateKey(privKey).PubKey()
		if sig.Verify(hash, pubKey) {
			t.Errorf("%s: unexpected success for invalid signature: %x",
				test.name, sig.Serialize())
			continue
		}
	}
}

// TestSignatureIsEqual ensures that equality testing between two signatures
// works as expected.
func TestSignatureIsEqual(t *testing.T) {
	sig1 := &Signature{
		r: *hexToModNScalar("82235e21a2300022738dabb8e1bbd9d19cfb1e7ab8c30a23b0afbb8d178abcf3"),
		s: *hexToModNScalar("24bf68e256c534ddfaf966bf908deb944305596f7bdcc38d69acad7f9c868724"),
	}
	sig1Copy := &Signature{
		r: *hexToModNScalar("82235e21a2300022738dabb8e1bbd9d19cfb1e7ab8c30a23b0afbb8d178abcf3"),
		s: *hexToModNScalar("24bf68e256c534ddfaf966bf908deb944305596f7bdcc38d69acad7f9c868724"),
	}
	sig2 := &Signature{
		r: *hexToModNScalar("4e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd41"),
		s: *hexToModNScalar("181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d09"),
	}

	if !sig1.IsEqual(sig1) {
		t.Fatalf("bad self signature equality check: %v == %v", sig1, sig1Copy)
	}
	if !sig1.IsEqual(sig1Copy) {
		t.Fatalf("bad signature equality check: %v == %v", sig1, sig1Copy)
	}

	if sig1.IsEqual(sig2) {
		t.Fatalf("bad signature equality check: %v != %v", sig1, sig2)
	}
}

// TestSignAndRecoverCompact ensures compact (recoverable public key) ECDSA
// signing and public key recovery works as expected for a selected set of
// private keys, messages, and nonces that have been verified independently with
// the Sage computer algebra system.
func TestSignAndRecoverCompact(t *testing.T) {
	t.Parallel()

	tests := signTests(t)
	for _, test := range tests {
		// Skip tests using nonces that are not RFC6979.
		if !test.rfc6979 {
			continue
		}

		// Parse test data.
		privKey := secp256k1.NewPrivateKey(hexToModNScalar(test.key))
		pubKey := privKey.PubKey()
		hash := hexToBytes(test.hash)
		wantSig := hexToBytes("00" + test.wantSigR + test.wantSigS)

		// Test compact signatures for both the compressed and uncompressed
		// versions of the public key.
		for _, compressed := range []bool{true, false} {
			// Populate the expected compact signature recovery code.
			wantRecoveryCode := compactSigMagicOffset + test.wantCode
			if compressed {
				wantRecoveryCode += compactSigCompPubKey
			}
			wantSig[0] = wantRecoveryCode

			// Sign the hash of the message with the given private key and
			// ensure the generated signature is the expected value per the
			// specified compressed flag.
			gotSig := SignCompact(privKey, hash, compressed)
			if !bytes.Equal(gotSig, wantSig) {
				t.Errorf("%s: unexpected signature -- got %x, want %x",
					test.name, gotSig, wantSig)
				continue
			}

			// Ensure the recovered public key and flag that indicates whether
			// or not the signature was for a compressed public key are the
			// expected values.
			gotPubKey, gotCompressed, err := RecoverCompact(gotSig, hash)
			if err != nil {
				t.Errorf("%s: unexpected error when recovering: %v", test.name,
					err)
				continue
			}
			if gotCompressed != compressed {
				t.Errorf("%s: unexpected compressed flag -- got %v, want %v",
					test.name, gotCompressed, compressed)
				continue
			}
			if !gotPubKey.IsEqual(pubKey) {
				t.Errorf("%s: unexpected public key -- got %x, want %x",
					test.name, gotPubKey.SerializeUncompressed(),
					pubKey.SerializeUncompressed())
				continue
			}
		}
	}
}

// TestRecoverCompactErrors ensures several error paths in compact signature
// recovery are detected as expected.  When possible, the signatures are
// otherwise valid with the exception of the specific failure to ensure it's
// robust against things like fault attacks.
func TestRecoverCompactErrors(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name string // test description
		sig  string // hex encoded signature to recover pubkey from
		hash string // hex encoded hash of message
		err  error  // expected error
	}{{
		name: "empty signature",
		sig:  "",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigInvalidLen,
	}, {
		// Signature created from private key 0x02, blake256(0x01020304).
		name: "no compact sig recovery code (otherwise valid sig)",
		sig: "e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigInvalidLen,
	}, {
		// Signature created from private key 0x02, blake256(0x01020304).
		name: "signature one byte too long (S padded with leading zero)",
		sig: "1f" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"0044b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigInvalidLen,
	}, {
		// Signature created from private key 0x02, blake256(0x01020304).
		name: "compact sig recovery code too low (otherwise valid sig)",
		sig: "1a" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigInvalidRecoveryCode,
	}, {
		// Signature created from private key 0x02, blake256(0x01020304).
		name: "compact sig recovery code too high (otherwise valid sig)",
		sig: "23" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigInvalidRecoveryCode,
	}, {
		// Signature invented since finding a signature with an r value that is
		// exactly the group order prior to the modular reduction is not
		// calculable without breaking the underlying crypto.
		name: "R == group order",
		sig: "1f" +
			"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigRTooBig,
	}, {
		// Signature invented since finding a signature with an r value that
		// would be valid modulo the group order and is still 32 bytes is not
		// calculable without breaking the underlying crypto.
		name: "R > group order and still 32 bytes (order + 1)",
		sig: "1f" +
			"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigRTooBig,
	}, {
		// Signature invented since the only way a signature could have an r
		// value of zero is if the nonce were zero which is invalid.
		name: "R == 0",
		sig: "1f" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigRIsZero,
	}, {
		// Signature invented since finding a signature with an s value that is
		// exactly the group order prior to the modular reduction is not
		// calculable without breaking the underlying crypto.
		name: "S == group order",
		sig: "1f" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigSTooBig,
	}, {
		// Signature invented since finding a signature with an s value that
		// would be valid modulo the group order and is still 32 bytes is not
		// calculable without breaking the underlying crypto.
		name: "S > group order and still 32 bytes (order + 1)",
		sig: "1f" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigSTooBig,
	}, {
		// Signature created by forcing the key/hash/nonce choices such that s
		// is zero and is therefore invalid.  The signing code will not produce
		// such a signature in practice.
		name: "S == 0",
		sig: "1f" +
			"e6f137b52377250760cc702e19b7aee3c63b0e7d95a91939b14ab3b5c4771e59" +
			"0000000000000000000000000000000000000000000000000000000000000000",
		hash: "393bec84f1a04037751c0d6c2817f37953eaa204ac0898de7adb038c33a20438",
		err:  ErrSigSIsZero,
	}, {
		// Signature invented since finding a private key needed to create a
		// valid signature with an r value that is >= group order prior to the
		// modular reduction is not possible without breaking the underlying
		// crypto.
		name: "R >= field prime minus group order with overflow bit",
		sig: "21" +
			"000000000000000000000000000000014551231950b75fc4402da1722fc9baee" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrSigOverflowsPrime,
	}, {
		// Signature invented since finding a private key needed to create a
		// valid signature with an r value that is > group order prior to the
		// modular reduction is not possible without breaking the underlying
		// crypto.
		name: "R > group order with overflow bit",
		sig: "21" +
			"000000000000000000000000000000014551231950b75fc4402da1722fc9baed" +
			"44b9bc4620afa158b7efdfea5234ff2d5f2f78b42886f02cf581827ee55318ea",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrPointNotOnCurve,
	}, {
		// Signature created from private key 0x01, blake256(0x0102030407) over
		// the secp256r1 curve (note the r1 instead of k1).
		name: "pubkey not on the curve, signature valid for secp256r1 instead",
		sig: "1f" +
			"2a81d1b3facc22185267d3f8832c5104902591bc471253f1cfc5eb25f4f740f2" +
			"72e65d019f9b09d769149e2be0b55de9b0224d34095bddc6a5dba90bfda33c45",
		hash: "9165e957708bc95cf62d020769c150b2d7b08e7ab7981860815b1eaabd41d695",
		err:  ErrPointNotOnCurve,
	}, {
		// Signature created from private key 0x01, blake256(0x01020304) and
		// manually setting s = -e*k^-1.
		name: "calculated pubkey point at infinity",
		sig: "1f" +
			"c6c4137b0e5fbfc88ae3f293d7e80c8566c43ae20340075d44f75b009c943d09" +
			"1281d8d90a5774045abd57b453c7eadbc830dbadec89ae8dd7639b9cc55641d0",
		hash: "c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7",
		err:  ErrPointNotOnCurve,
	}}

	for _, test := range tests {
		// Parse test data.
		hash := hexToBytes(test.hash)
		sig := hexToBytes(test.sig)

		// Ensure the expected error is hit.
		_, _, err := RecoverCompact(sig, hash)
		if !errors.Is(err, test.err) {
			t.Errorf("%s: mismatched err -- got %v, want %v", test.name, err,
				test.err)
			continue
		}
	}
}

// TestSignAndRecoverCompactRandom ensures compact (recoverable public key)
// ECDSA signing and recovery work as expected for randomly-generated private
// keys and messages.  It also ensures mutated signatures and messages do not
// improperly recover the original public key.
func TestSignAndRecoverCompactRandom(t *testing.T) {
	t.Parallel()

	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := rand.New(rand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	for i := 0; i < 100; i++ {
		// Generate a random private key.
		var buf [32]byte
		if _, err := rng.Read(buf[:]); err != nil {
			t.Fatalf("failed to read random private key: %v", err)
		}
		var privKeyScalar secp256k1.ModNScalar
		privKeyScalar.SetBytes(&buf)
		privKey := secp256k1.NewPrivateKey(&privKeyScalar)
		wantPubKey := privKey.PubKey()

		// Generate a random hash to sign.
		var hash [32]byte
		if _, err := rng.Read(hash[:]); err != nil {
			t.Fatalf("failed to read random hash: %v", err)
		}

		// Test compact signatures for both the compressed and uncompressed
		// versions of the public key.
		for _, compressed := range []bool{true, false} {
			// Sign the hash with the private key and then ensure the original
			// public key and compressed flag is recovered from the produced
			// signature.
			gotSig := SignCompact(privKey, hash[:], compressed)

			gotPubKey, gotCompressed, err := RecoverCompact(gotSig, hash[:])
			if err != nil {
				t.Fatalf("unexpected err: %v\nsig: %x\nhash: %x\nprivate key: %x",
					err, gotSig, hash, privKey.Serialize())
			}
			if gotCompressed != compressed {
				t.Fatalf("unexpected compressed flag: %v\nsig: %x\nhash: %x\n"+
					"private key: %x", gotCompressed, gotSig, hash,
					privKey.Serialize())
			}
			if !gotPubKey.IsEqual(wantPubKey) {
				t.Fatalf("unexpected recovered public key: %x\nsig: %x\nhash: "+
					"%x\nprivate key: %x", gotPubKey.SerializeUncompressed(),
					gotSig, hash, privKey.Serialize())
			}

			// Change a random bit in the signature and ensure the bad signature
			// fails to recover the original public key.
			badSig := make([]byte, len(gotSig))
			copy(badSig, gotSig)
			randByte := rng.Intn(len(badSig)-1) + 1
			randBit := rng.Intn(7)
			badSig[randByte] ^= 1 << randBit
			badPubKey, _, err := RecoverCompact(badSig, hash[:])
			if err == nil && badPubKey.IsEqual(wantPubKey) {
				t.Fatalf("recovered public key for bad sig: %x\nhash: %x\n"+
					"private key: %x", badSig, hash, privKey.Serialize())
			}

			// Change a random bit in the hash that was originally signed and
			// ensure the original good signature fails to recover the original
			// public key.
			badHash := make([]byte, len(hash))
			copy(badHash, hash[:])
			randByte = rng.Intn(len(badHash))
			randBit = rng.Intn(7)
			badHash[randByte] ^= 1 << randBit
			badPubKey, _, err = RecoverCompact(gotSig, badHash)
			if err == nil && badPubKey.IsEqual(wantPubKey) {
				t.Fatalf("recovered public key for bad hash: %x\nsig: %x\n"+
					"private key: %x", badHash, gotSig, privKey.Serialize())
			}
		}
	}
}
