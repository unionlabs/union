// Copyright (c) 2013-2016 The btcsuite developers
// Copyright (c) 2015-2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"bytes"
	"errors"
	"testing"
)

// TestParsePubKey ensures that public keys are properly parsed according
// to the spec including both the positive and negative cases.
func TestParsePubKey(t *testing.T) {
	tests := []struct {
		name  string // test description
		key   string // hex encoded public key
		err   error  // expected error
		wantX string // expected x coordinate
		wantY string // expected y coordinate
	}{{
		name: "uncompressed ok",
		key: "04" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err:   nil,
		wantX: "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		wantY: "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
	}, {
		name: "uncompressed x changed (not on curve)",
		key: "04" +
			"15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err: ErrPubKeyNotOnCurve,
	}, {
		name: "uncompressed y changed (not on curve)",
		key: "04" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a4",
		err: ErrPubKeyNotOnCurve,
	}, {
		name: "uncompressed claims compressed",
		key: "03" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err: ErrPubKeyInvalidFormat,
	}, {
		name: "uncompressed as hybrid ok (ybit = 0)",
		key: "06" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"4d1f1522047b33068bbb9b07d1e9f40564749b062b3fc0666479bc08a94be98c",
		err:   nil,
		wantX: "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		wantY: "4d1f1522047b33068bbb9b07d1e9f40564749b062b3fc0666479bc08a94be98c",
	}, {
		name: "uncompressed as hybrid ok (ybit = 1)",
		key: "07" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err:   nil,
		wantX: "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		wantY: "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
	}, {
		name: "uncompressed as hybrid wrong oddness",
		key: "06" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err: ErrPubKeyMismatchedOddness,
	}, {
		name: "compressed ok (ybit = 0)",
		key: "02" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
		err:   nil,
		wantX: "ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
		wantY: "0890ff84d7999d878a57bee170e19ef4b4803b4bdede64503a6ac352b03c8032",
	}, {
		name: "compressed ok (ybit = 1)",
		key: "03" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
		err:   nil,
		wantX: "2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
		wantY: "499dd7852849a38aa23ed9f306f07794063fe7904e0f347bc209fdddaf37691f",
	}, {
		name: "compressed claims uncompressed (ybit = 0)",
		key: "04" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
		err: ErrPubKeyInvalidFormat,
	}, {
		name: "compressed claims uncompressed (ybit = 1)",
		key: "04" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
		err: ErrPubKeyInvalidFormat,
	}, {
		name: "compressed claims hybrid (ybit = 0)",
		key: "06" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
		err: ErrPubKeyInvalidFormat,
	}, {
		name: "compressed claims hybrid (ybit = 1)",
		key: "07" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
		err: ErrPubKeyInvalidFormat,
	}, {
		name: "compressed with invalid x coord (ybit = 0)",
		key: "03" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4c",
		err: ErrPubKeyNotOnCurve,
	}, {
		name: "compressed with invalid x coord (ybit = 1)",
		key: "03" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448d",
		err: ErrPubKeyNotOnCurve,
	}, {
		name: "empty",
		key:  "",
		err:  ErrPubKeyInvalidLen,
	}, {
		name: "wrong length",
		key:  "05",
		err:  ErrPubKeyInvalidLen,
	}, {
		name: "uncompressed x == p",
		key: "04" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err: ErrPubKeyXTooBig,
	}, {
		// The y coordinate produces a valid point for x == 1 (mod p), but it
		// should fail to parse instead of wrapping around.
		name: "uncompressed x > p (p + 1 -- aka 1)",
		key: "04" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30" +
			"bde70df51939b94c9c24979fa7dd04ebd9b3572da7802290438af2a681895441",
		err: ErrPubKeyXTooBig,
	}, {
		name: "uncompressed y == p",
		key: "04" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		err: ErrPubKeyYTooBig,
	}, {
		// The x coordinate produces a valid point for y == 1 (mod p), but it
		// should fail to parse instead of wrapping around.
		name: "uncompressed y > p (p + 1 -- aka 1)",
		key: "04" +
			"1fe1e5ef3fceb5c135ab7741333ce5a6e80d68167653f6b2b24bcbcfaaaff507" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30",
		err: ErrPubKeyYTooBig,
	}, {
		name: "compressed x == p (ybit = 0)",
		key: "02" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		err: ErrPubKeyXTooBig,
	}, {
		name: "compressed x == p (ybit = 1)",
		key: "03" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		err: ErrPubKeyXTooBig,
	}, {
		// This would be valid for x == 2 (mod p), but it should fail to parse
		// instead of wrapping around.
		name: "compressed x > p (p + 2 -- aka 2) (ybit = 0)",
		key: "02" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc31",
		err: ErrPubKeyXTooBig,
	}, {
		// This would be valid for x == 1 (mod p), but it should fail to parse
		// instead of wrapping around.
		name: "compressed x > p (p + 1 -- aka 1) (ybit = 1)",
		key: "03" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30",
		err: ErrPubKeyXTooBig,
	}, {
		name: "hybrid x == p (ybit = 1)",
		key: "07" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		err: ErrPubKeyXTooBig,
	}, {
		// The y coordinate produces a valid point for x == 1 (mod p), but it
		// should fail to parse instead of wrapping around.
		name: "hybrid x > p (p + 1 -- aka 1) (ybit = 0)",
		key: "06" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30" +
			"bde70df51939b94c9c24979fa7dd04ebd9b3572da7802290438af2a681895441",
		err: ErrPubKeyXTooBig,
	}, {
		name: "hybrid y == p (ybit = 0 when mod p)",
		key: "06" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		err: ErrPubKeyYTooBig,
	}, {
		// The x coordinate produces a valid point for y == 1 (mod p), but it
		// should fail to parse instead of wrapping around.
		name: "hybrid y > p (p + 1 -- aka 1) (ybit = 1 when mod p)",
		key: "07" +
			"1fe1e5ef3fceb5c135ab7741333ce5a6e80d68167653f6b2b24bcbcfaaaff507" +
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30",
		err: ErrPubKeyYTooBig,
	}}

	for _, test := range tests {
		pubKeyBytes := hexToBytes(test.key)
		pubKey, err := ParsePubKey(pubKeyBytes)
		if !errors.Is(err, test.err) {
			t.Errorf("%s mismatched err -- got %v, want %v", test.name, err,
				test.err)
			continue
		}
		if err != nil {
			continue
		}

		// Ensure the x and y coordinates match the expected values upon
		// successful parse.
		wantX, wantY := hexToFieldVal(test.wantX), hexToFieldVal(test.wantY)
		if !pubKey.x.Equals(wantX) {
			t.Errorf("%s: mismatched x coordinate -- got %v, want %v",
				test.name, pubKey.x, wantX)
			continue
		}
		if !pubKey.y.Equals(wantY) {
			t.Errorf("%s: mismatched y coordinate -- got %v, want %v",
				test.name, pubKey.y, wantY)
			continue
		}
	}
}

// TestPubKeySerialize ensures that serializing public keys works as expected
// for both the compressed and uncompressed cases.
func TestPubKeySerialize(t *testing.T) {
	tests := []struct {
		name     string // test description
		pubX     string // hex encoded x coordinate for pubkey to serialize
		pubY     string // hex encoded y coordinate for pubkey to serialize
		compress bool   // whether to serialize compressed or uncompressed
		expected string // hex encoded expected pubkey serialization
	}{{
		name:     "uncompressed (ybit = 0)",
		pubX:     "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY:     "4d1f1522047b33068bbb9b07d1e9f40564749b062b3fc0666479bc08a94be98c",
		compress: false,
		expected: "04" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"4d1f1522047b33068bbb9b07d1e9f40564749b062b3fc0666479bc08a94be98c",
	}, {
		name:     "uncompressed (ybit = 1)",
		pubX:     "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY:     "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		compress: false,
		expected: "04" +
			"11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
	}, {
		// It's invalid to parse pubkeys that are not on the curve, however it
		// is possible to manually create them and they should serialize
		// correctly.
		name:     "uncompressed not on the curve due to x coord",
		pubX:     "15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY:     "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		compress: false,
		expected: "04" +
			"15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
	}, {
		// It's invalid to parse pubkeys that are not on the curve, however it
		// is possible to manually create them and they should serialize
		// correctly.
		name:     "uncompressed not on the curve due to y coord",
		pubX:     "15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY:     "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a4",
		compress: false,
		expected: "04" +
			"15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c" +
			"b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a4",
	}, {
		name:     "compressed (ybit = 0)",
		pubX:     "ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
		pubY:     "0890ff84d7999d878a57bee170e19ef4b4803b4bdede64503a6ac352b03c8032",
		compress: true,
		expected: "02" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d",
	}, {
		name:     "compressed (ybit = 1)",
		pubX:     "2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
		pubY:     "499dd7852849a38aa23ed9f306f07794063fe7904e0f347bc209fdddaf37691f",
		compress: true,
		expected: "03" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e",
	}, {
		// It's invalid to parse pubkeys that are not on the curve, however it
		// is possible to manually create them and they should serialize
		// correctly.
		name:     "compressed not on curve (ybit = 0)",
		pubX:     "ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4c",
		pubY:     "0890ff84d7999d878a57bee170e19ef4b4803b4bdede64503a6ac352b03c8032",
		compress: true,
		expected: "02" +
			"ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4c",
	}, {
		// It's invalid to parse pubkeys that are not on the curve, however it
		// is possible to manually create them and they should serialize
		// correctly.
		name:     "compressed not on curve (ybit = 1)",
		pubX:     "2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448d",
		pubY:     "499dd7852849a38aa23ed9f306f07794063fe7904e0f347bc209fdddaf37691f",
		compress: true,
		expected: "03" +
			"2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448d",
	}}

	for _, test := range tests {
		// Parse the test data.
		x, y := hexToFieldVal(test.pubX), hexToFieldVal(test.pubY)
		pubKey := NewPublicKey(x, y)

		// Serialize with the correct method and ensure the result matches the
		// expected value.
		var serialized []byte
		if test.compress {
			serialized = pubKey.SerializeCompressed()
		} else {
			serialized = pubKey.SerializeUncompressed()
		}
		expected := hexToBytes(test.expected)
		if !bytes.Equal(serialized, expected) {
			t.Errorf("%s: mismatched serialized public key -- got %x, want %x",
				test.name, serialized, expected)
			continue
		}
	}
}

// TestPublicKeyIsEqual ensures that equality testing between two public keys
// works as expected.
func TestPublicKeyIsEqual(t *testing.T) {
	pubKey1 := &PublicKey{
		x: *hexToFieldVal("2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e"),
		y: *hexToFieldVal("499dd7852849a38aa23ed9f306f07794063fe7904e0f347bc209fdddaf37691f"),
	}
	pubKey1Copy := &PublicKey{
		x: *hexToFieldVal("2689c7c2dab13309fb143e0e8fe396342521887e976690b6b47f5b2a4b7d448e"),
		y: *hexToFieldVal("499dd7852849a38aa23ed9f306f07794063fe7904e0f347bc209fdddaf37691f"),
	}
	pubKey2 := &PublicKey{
		x: *hexToFieldVal("ce0b14fb842b1ba549fdd675c98075f12e9c510f8ef52bd021a9a1f4809d3b4d"),
		y: *hexToFieldVal("0890ff84d7999d878a57bee170e19ef4b4803b4bdede64503a6ac352b03c8032"),
	}

	if !pubKey1.IsEqual(pubKey1) {
		t.Fatalf("bad self public key equality check: (%v, %v)", pubKey1.x,
			pubKey1.y)
	}
	if !pubKey1.IsEqual(pubKey1Copy) {
		t.Fatalf("bad public key equality check: (%v, %v) == (%v, %v)",
			pubKey1.x, pubKey1.y, pubKey1Copy.x, pubKey1Copy.y)
	}

	if pubKey1.IsEqual(pubKey2) {
		t.Fatalf("bad public key equality check: (%v, %v) != (%v, %v)",
			pubKey1.x, pubKey1.y, pubKey2.x, pubKey2.y)
	}
}

// TestPublicKeyAsJacobian ensures converting a public key to a jacobian point
// with a Z coordinate of 1 works as expected.
func TestPublicKeyAsJacobian(t *testing.T) {
	tests := []struct {
		name   string // test description
		pubKey string // hex encoded serialized compressed pubkey
		wantX  string // hex encoded expected X coordinate
		wantY  string // hex encoded expected Y coordinate

	}{{
		name:   "public key for private key 0x01",
		pubKey: "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
		wantX:  "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
		wantY:  "483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
	}, {
		name:   "public for private key 0x03",
		pubKey: "02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
		wantX:  "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
		wantY:  "388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672",
	}, {
		name:   "public for private key 0x06",
		pubKey: "03fff97bd5755eeea420453a14355235d382f6472f8568a18b2f057a1460297556",
		wantX:  "fff97bd5755eeea420453a14355235d382f6472f8568a18b2f057a1460297556",
		wantY:  "ae12777aacfbb620f3be96017f45c560de80f0f6518fe4a03c870c36b075f297",
	}}

	for _, test := range tests {
		// Parse the test data.
		pubKeyBytes := hexToBytes(test.pubKey)
		wantX := hexToFieldVal(test.wantX)
		wantY := hexToFieldVal(test.wantY)
		pubKey, err := ParsePubKey(pubKeyBytes)
		if err != nil {
			t.Errorf("%s: failed to parse public key: %v", test.name, err)
			continue
		}

		// Convert the public key to a jacobian point and ensure the coordinates
		// match the expected values.
		var point JacobianPoint
		pubKey.AsJacobian(&point)
		if !point.Z.IsOne() {
			t.Errorf("%s: invalid Z coordinate -- got %v, want 1", test.name,
				point.Z)
			continue
		}
		if !point.X.Equals(wantX) {
			t.Errorf("%s: invalid X coordinate - got %v, want %v", test.name,
				point.X, wantX)
			continue
		}
		if !point.Y.Equals(wantY) {
			t.Errorf("%s: invalid Y coordinate - got %v, want %v", test.name,
				point.Y, wantY)
			continue
		}
	}
}

// TestPublicKeyIsOnCurve ensures testing if a public key is on the curve works
// as expected.
func TestPublicKeyIsOnCurve(t *testing.T) {
	tests := []struct {
		name string // test description
		pubX string // hex encoded x coordinate for pubkey to serialize
		pubY string // hex encoded y coordinate for pubkey to serialize
		want bool   // expected result
	}{{
		name: "valid with even y",
		pubX: "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY: "4d1f1522047b33068bbb9b07d1e9f40564749b062b3fc0666479bc08a94be98c",
		want: true,
	}, {
		name: "valid with odd y",
		pubX: "11db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY: "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		want: true,
	}, {
		name: "invalid due to x coord",
		pubX: "15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY: "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3",
		want: false,
	}, {
		name: "invalid due to y coord",
		pubX: "15db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5c",
		pubY: "b2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a4",
		want: false,
	}}

	for _, test := range tests {
		// Parse the test data.
		x, y := hexToFieldVal(test.pubX), hexToFieldVal(test.pubY)
		pubKey := NewPublicKey(x, y)

		result := pubKey.IsOnCurve()
		if result != test.want {
			t.Errorf("%s: mismatched is on curve result -- got %v, want %v",
				test.name, result, test.want)
			continue
		}
	}
}
