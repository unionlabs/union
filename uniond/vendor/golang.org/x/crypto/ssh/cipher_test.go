// Copyright 2011 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"bytes"
	"crypto"
	"crypto/rand"
	"encoding/binary"
	"io"
	"testing"

	"golang.org/x/crypto/chacha20"
	"golang.org/x/crypto/internal/poly1305"
)

func TestDefaultCiphersExist(t *testing.T) {
	for _, cipherAlgo := range supportedCiphers {
		if _, ok := cipherModes[cipherAlgo]; !ok {
			t.Errorf("supported cipher %q is unknown", cipherAlgo)
		}
	}
	for _, cipherAlgo := range preferredCiphers {
		if _, ok := cipherModes[cipherAlgo]; !ok {
			t.Errorf("preferred cipher %q is unknown", cipherAlgo)
		}
	}
}

func TestPacketCiphers(t *testing.T) {
	defaultMac := "hmac-sha2-256"
	defaultCipher := "aes128-ctr"
	for cipher := range cipherModes {
		t.Run("cipher="+cipher,
			func(t *testing.T) { testPacketCipher(t, cipher, defaultMac) })
	}
	for mac := range macModes {
		t.Run("mac="+mac,
			func(t *testing.T) { testPacketCipher(t, defaultCipher, mac) })
	}
}

func testPacketCipher(t *testing.T, cipher, mac string) {
	kr := &kexResult{Hash: crypto.SHA1}
	algs := directionAlgorithms{
		Cipher:      cipher,
		MAC:         mac,
		Compression: "none",
	}
	client, err := newPacketCipher(clientKeys, algs, kr)
	if err != nil {
		t.Fatalf("newPacketCipher(client, %q, %q): %v", cipher, mac, err)
	}
	server, err := newPacketCipher(clientKeys, algs, kr)
	if err != nil {
		t.Fatalf("newPacketCipher(client, %q, %q): %v", cipher, mac, err)
	}

	want := "bla bla"
	input := []byte(want)
	buf := &bytes.Buffer{}
	if err := client.writeCipherPacket(0, buf, rand.Reader, input); err != nil {
		t.Fatalf("writeCipherPacket(%q, %q): %v", cipher, mac, err)
	}

	packet, err := server.readCipherPacket(0, buf)
	if err != nil {
		t.Fatalf("readCipherPacket(%q, %q): %v", cipher, mac, err)
	}

	if string(packet) != want {
		t.Errorf("roundtrip(%q, %q): got %q, want %q", cipher, mac, packet, want)
	}
}

func TestCBCOracleCounterMeasure(t *testing.T) {
	kr := &kexResult{Hash: crypto.SHA1}
	algs := directionAlgorithms{
		Cipher:      aes128cbcID,
		MAC:         "hmac-sha1",
		Compression: "none",
	}
	client, err := newPacketCipher(clientKeys, algs, kr)
	if err != nil {
		t.Fatalf("newPacketCipher(client): %v", err)
	}

	want := "bla bla"
	input := []byte(want)
	buf := &bytes.Buffer{}
	if err := client.writeCipherPacket(0, buf, rand.Reader, input); err != nil {
		t.Errorf("writeCipherPacket: %v", err)
	}

	packetSize := buf.Len()
	buf.Write(make([]byte, 2*maxPacket))

	// We corrupt each byte, but this usually will only test the
	// 'packet too large' or 'MAC failure' cases.
	lastRead := -1
	for i := 0; i < packetSize; i++ {
		server, err := newPacketCipher(clientKeys, algs, kr)
		if err != nil {
			t.Fatalf("newPacketCipher(client): %v", err)
		}

		fresh := &bytes.Buffer{}
		fresh.Write(buf.Bytes())
		fresh.Bytes()[i] ^= 0x01

		before := fresh.Len()
		_, err = server.readCipherPacket(0, fresh)
		if err == nil {
			t.Errorf("corrupt byte %d: readCipherPacket succeeded ", i)
			continue
		}
		if _, ok := err.(cbcError); !ok {
			t.Errorf("corrupt byte %d: got %v (%T), want cbcError", i, err, err)
			continue
		}

		after := fresh.Len()
		bytesRead := before - after
		if bytesRead < maxPacket {
			t.Errorf("corrupt byte %d: read %d bytes, want more than %d", i, bytesRead, maxPacket)
			continue
		}

		if i > 0 && bytesRead != lastRead {
			t.Errorf("corrupt byte %d: read %d bytes, want %d bytes read", i, bytesRead, lastRead)
		}
		lastRead = bytesRead
	}
}

func TestCVE202143565(t *testing.T) {
	tests := []struct {
		cipher          string
		constructPacket func(packetCipher) io.Reader
	}{
		{
			cipher: gcm128CipherID,
			constructPacket: func(client packetCipher) io.Reader {
				internalCipher := client.(*gcmCipher)
				b := &bytes.Buffer{}
				prefix := [4]byte{}
				if _, err := b.Write(prefix[:]); err != nil {
					t.Fatal(err)
				}
				internalCipher.buf = internalCipher.aead.Seal(internalCipher.buf[:0], internalCipher.iv, []byte{}, prefix[:])
				if _, err := b.Write(internalCipher.buf); err != nil {
					t.Fatal(err)
				}
				internalCipher.incIV()

				return b
			},
		},
		{
			cipher: chacha20Poly1305ID,
			constructPacket: func(client packetCipher) io.Reader {
				internalCipher := client.(*chacha20Poly1305Cipher)
				b := &bytes.Buffer{}

				nonce := make([]byte, 12)
				s, err := chacha20.NewUnauthenticatedCipher(internalCipher.contentKey[:], nonce)
				if err != nil {
					t.Fatal(err)
				}
				var polyKey, discardBuf [32]byte
				s.XORKeyStream(polyKey[:], polyKey[:])
				s.XORKeyStream(discardBuf[:], discardBuf[:]) // skip the next 32 bytes

				internalCipher.buf = make([]byte, 4+poly1305.TagSize)
				binary.BigEndian.PutUint32(internalCipher.buf, 0)
				ls, err := chacha20.NewUnauthenticatedCipher(internalCipher.lengthKey[:], nonce)
				if err != nil {
					t.Fatal(err)
				}
				ls.XORKeyStream(internalCipher.buf, internalCipher.buf[:4])
				if _, err := io.ReadFull(rand.Reader, internalCipher.buf[4:4]); err != nil {
					t.Fatal(err)
				}

				s.XORKeyStream(internalCipher.buf[4:], internalCipher.buf[4:4])

				var tag [poly1305.TagSize]byte
				poly1305.Sum(&tag, internalCipher.buf[:4], &polyKey)

				copy(internalCipher.buf[4:], tag[:])

				if _, err := b.Write(internalCipher.buf); err != nil {
					t.Fatal(err)
				}

				return b
			},
		},
	}

	for _, tc := range tests {
		mac := "hmac-sha2-256"

		kr := &kexResult{Hash: crypto.SHA1}
		algs := directionAlgorithms{
			Cipher:      tc.cipher,
			MAC:         mac,
			Compression: "none",
		}
		client, err := newPacketCipher(clientKeys, algs, kr)
		if err != nil {
			t.Fatalf("newPacketCipher(client, %q, %q): %v", tc.cipher, mac, err)
		}
		server, err := newPacketCipher(clientKeys, algs, kr)
		if err != nil {
			t.Fatalf("newPacketCipher(client, %q, %q): %v", tc.cipher, mac, err)
		}

		b := tc.constructPacket(client)

		wantErr := "ssh: empty packet"
		_, err = server.readCipherPacket(0, b)
		if err == nil {
			t.Fatalf("readCipherPacket(%q, %q): didn't fail with empty packet", tc.cipher, mac)
		} else if err.Error() != wantErr {
			t.Fatalf("readCipherPacket(%q, %q): unexpected error, got %q, want %q", tc.cipher, mac, err, wantErr)
		}
	}
}
