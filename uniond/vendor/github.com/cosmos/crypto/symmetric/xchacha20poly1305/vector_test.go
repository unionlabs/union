package xchacha20poly1305

import (
	"bytes"
	"encoding/hex"
	"testing"
)

func toHex(bits []byte) string {
	return hex.EncodeToString(bits)
}

func fromHex(bits string) []byte {
	b, err := hex.DecodeString(bits)
	if err != nil {
		panic(err)
	}
	return b
}

func TestHChaCha20(t *testing.T) {
	for i, v := range hChaCha20Vectors {
		var key [32]byte
		var nonce [16]byte
		copy(key[:], v.key)
		copy(nonce[:], v.nonce)

		HChaCha20(&key, &nonce, &key)
		if !bytes.Equal(key[:], v.keystream) {
			t.Errorf("test %d: keystream mismatch:\n \t got:  %s\n \t want: %s", i, toHex(key[:]), toHex(v.keystream))
		}
	}
}

var hChaCha20Vectors = []struct {
	key, nonce, keystream []byte
}{
	{
		fromHex("0000000000000000000000000000000000000000000000000000000000000000"),
		fromHex("000000000000000000000000000000000000000000000000"),
		fromHex("1140704c328d1d5d0e30086cdf209dbd6a43b8f41518a11cc387b669b2ee6586"),
	},
	{
		fromHex("8000000000000000000000000000000000000000000000000000000000000000"),
		fromHex("000000000000000000000000000000000000000000000000"),
		fromHex("7d266a7fd808cae4c02a0a70dcbfbcc250dae65ce3eae7fc210f54cc8f77df86"),
	},
	{
		fromHex("0000000000000000000000000000000000000000000000000000000000000001"),
		fromHex("000000000000000000000000000000000000000000000002"),
		fromHex("e0c77ff931bb9163a5460c02ac281c2b53d792b1c43fea817e9ad275ae546963"),
	},
	{
		fromHex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"),
		fromHex("000102030405060708090a0b0c0d0e0f1011121314151617"),
		fromHex("51e3ff45a895675c4b33b46c64f4a9ace110d34df6a2ceab486372bacbd3eff6"),
	},
	{
		fromHex("24f11cce8a1b3d61e441561a696c1c1b7e173d084fd4812425435a8896a013dc"),
		fromHex("d9660c5900ae19ddad28d6e06e45fe5e"),
		fromHex("5966b3eec3bff1189f831f06afe4d4e3be97fa9235ec8c20d08acfbbb4e851e3"),
	},
}

func TestVectors(t *testing.T) {
	for i, v := range vectors {
		if len(v.plaintext) == 0 {
			v.plaintext = make([]byte, len(v.ciphertext))
		}

		var nonce [24]byte
		copy(nonce[:], v.nonce)

		aead, err := New(v.key)
		if err != nil {
			t.Error(err)
		}

		dst := aead.Seal(nil, nonce[:], v.plaintext, v.ad)
		if !bytes.Equal(dst, v.ciphertext) {
			t.Errorf("test %d: ciphertext mismatch:\n \t got:  %s\n \t want: %s", i, toHex(dst), toHex(v.ciphertext))
		}
		open, err := aead.Open(nil, nonce[:], dst, v.ad)
		if err != nil {
			t.Error(err)
		}
		if !bytes.Equal(open, v.plaintext) {
			t.Errorf("test %d: plaintext mismatch:\n \t got:  %s\n \t want: %s", i, string(open), string(v.plaintext))
		}
	}
}

var vectors = []struct {
	key, nonce, ad, plaintext, ciphertext []byte
}{
	{
		[]byte{
			0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a,
			0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95,
			0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
		},
		[]byte{0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b},
		[]byte{0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7},
		[]byte(
			"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.",
		),
		[]byte{
			0x45, 0x3c, 0x06, 0x93, 0xa7, 0x40, 0x7f, 0x04, 0xff, 0x4c, 0x56,
			0xae, 0xdb, 0x17, 0xa3, 0xc0, 0xa1, 0xaf, 0xff, 0x01, 0x17, 0x49,
			0x30, 0xfc, 0x22, 0x28, 0x7c, 0x33, 0xdb, 0xcf, 0x0a, 0xc8, 0xb8,
			0x9a, 0xd9, 0x29, 0x53, 0x0a, 0x1b, 0xb3, 0xab, 0x5e, 0x69, 0xf2,
			0x4c, 0x7f, 0x60, 0x70, 0xc8, 0xf8, 0x40, 0xc9, 0xab, 0xb4, 0xf6,
			0x9f, 0xbf, 0xc8, 0xa7, 0xff, 0x51, 0x26, 0xfa, 0xee, 0xbb, 0xb5,
			0x58, 0x05, 0xee, 0x9c, 0x1c, 0xf2, 0xce, 0x5a, 0x57, 0x26, 0x32,
			0x87, 0xae, 0xc5, 0x78, 0x0f, 0x04, 0xec, 0x32, 0x4c, 0x35, 0x14,
			0x12, 0x2c, 0xfc, 0x32, 0x31, 0xfc, 0x1a, 0x8b, 0x71, 0x8a, 0x62,
			0x86, 0x37, 0x30, 0xa2, 0x70, 0x2b, 0xb7, 0x63, 0x66, 0x11, 0x6b,
			0xed, 0x09, 0xe0, 0xfd, 0x5c, 0x6d, 0x84, 0xb6, 0xb0, 0xc1, 0xab,
			0xaf, 0x24, 0x9d, 0x5d, 0xd0, 0xf7, 0xf5, 0xa7, 0xea,
		},
	},
}
