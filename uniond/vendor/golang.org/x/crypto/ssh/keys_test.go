// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"bytes"
	"crypto/dsa"
	"crypto/ecdsa"
	"crypto/ed25519"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/hex"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"reflect"
	"strings"
	"testing"

	"golang.org/x/crypto/ssh/testdata"
)

func rawKey(pub PublicKey) interface{} {
	switch k := pub.(type) {
	case *rsaPublicKey:
		return (*rsa.PublicKey)(k)
	case *dsaPublicKey:
		return (*dsa.PublicKey)(k)
	case *ecdsaPublicKey:
		return (*ecdsa.PublicKey)(k)
	case ed25519PublicKey:
		return (ed25519.PublicKey)(k)
	case *Certificate:
		return k
	}
	panic("unknown key type")
}

func TestKeyMarshalParse(t *testing.T) {
	for _, priv := range testSigners {
		pub := priv.PublicKey()
		roundtrip, err := ParsePublicKey(pub.Marshal())
		if err != nil {
			t.Errorf("ParsePublicKey(%T): %v", pub, err)
		}

		k1 := rawKey(pub)
		k2 := rawKey(roundtrip)

		if !reflect.DeepEqual(k1, k2) {
			t.Errorf("got %#v in roundtrip, want %#v", k2, k1)
		}
	}
}

func TestUnsupportedCurves(t *testing.T) {
	raw, err := ecdsa.GenerateKey(elliptic.P224(), rand.Reader)
	if err != nil {
		t.Fatalf("GenerateKey: %v", err)
	}

	if _, err = NewSignerFromKey(raw); err == nil || !strings.Contains(err.Error(), "only P-256") {
		t.Fatalf("NewPrivateKey should not succeed with P-224, got: %v", err)
	}

	if _, err = NewPublicKey(&raw.PublicKey); err == nil || !strings.Contains(err.Error(), "only P-256") {
		t.Fatalf("NewPublicKey should not succeed with P-224, got: %v", err)
	}
}

func TestNewPublicKey(t *testing.T) {
	for _, k := range testSigners {
		raw := rawKey(k.PublicKey())
		// Skip certificates, as NewPublicKey does not support them.
		if _, ok := raw.(*Certificate); ok {
			continue
		}
		pub, err := NewPublicKey(raw)
		if err != nil {
			t.Errorf("NewPublicKey(%#v): %v", raw, err)
		}
		if !reflect.DeepEqual(k.PublicKey(), pub) {
			t.Errorf("NewPublicKey(%#v) = %#v, want %#v", raw, pub, k.PublicKey())
		}
	}
}

func TestKeySignVerify(t *testing.T) {
	for _, priv := range testSigners {
		pub := priv.PublicKey()

		data := []byte("sign me")
		sig, err := priv.Sign(rand.Reader, data)
		if err != nil {
			t.Fatalf("Sign(%T): %v", priv, err)
		}

		if err := pub.Verify(data, sig); err != nil {
			t.Errorf("publicKey.Verify(%T): %v", priv, err)
		}
		sig.Blob[5]++
		if err := pub.Verify(data, sig); err == nil {
			t.Errorf("publicKey.Verify on broken sig did not fail")
		}
	}
}

func TestKeySignWithAlgorithmVerify(t *testing.T) {
	for k, priv := range testSigners {
		if algorithmSigner, ok := priv.(MultiAlgorithmSigner); !ok {
			t.Errorf("Signers %q constructed by ssh package should always implement the MultiAlgorithmSigner interface: %T", k, priv)
		} else {
			pub := priv.PublicKey()
			data := []byte("sign me")

			signWithAlgTestCase := func(algorithm string, expectedAlg string) {
				sig, err := algorithmSigner.SignWithAlgorithm(rand.Reader, data, algorithm)
				if err != nil {
					t.Fatalf("Sign(%T): %v", priv, err)
				}
				if sig.Format != expectedAlg {
					t.Errorf("signature format did not match requested signature algorithm: %s != %s", sig.Format, expectedAlg)
				}

				if err := pub.Verify(data, sig); err != nil {
					t.Errorf("publicKey.Verify(%T): %v", priv, err)
				}
				sig.Blob[5]++
				if err := pub.Verify(data, sig); err == nil {
					t.Errorf("publicKey.Verify on broken sig did not fail")
				}
			}

			// Using the empty string as the algorithm name should result in the same signature format as the algorithm-free Sign method.
			defaultSig, err := priv.Sign(rand.Reader, data)
			if err != nil {
				t.Fatalf("Sign(%T): %v", priv, err)
			}
			signWithAlgTestCase("", defaultSig.Format)

			// RSA keys are the only ones which currently support more than one signing algorithm
			if pub.Type() == KeyAlgoRSA {
				for _, algorithm := range []string{KeyAlgoRSA, KeyAlgoRSASHA256, KeyAlgoRSASHA512} {
					signWithAlgTestCase(algorithm, algorithm)
				}
			}
		}
	}
}

func TestKeySignWithShortSignature(t *testing.T) {
	signer := testSigners["rsa"].(AlgorithmSigner)
	pub := signer.PublicKey()
	// Note: data obtained by empirically trying until a result
	// starting with 0 appeared
	tests := []struct {
		algorithm string
		data      []byte
	}{
		{
			algorithm: KeyAlgoRSA,
			data:      []byte("sign me92"),
		},
		{
			algorithm: KeyAlgoRSASHA256,
			data:      []byte("sign me294"),
		},
		{
			algorithm: KeyAlgoRSASHA512,
			data:      []byte("sign me60"),
		},
	}

	for _, tt := range tests {
		sig, err := signer.SignWithAlgorithm(rand.Reader, tt.data, tt.algorithm)
		if err != nil {
			t.Fatalf("Sign(%T): %v", signer, err)
		}
		if sig.Blob[0] != 0 {
			t.Errorf("%s: Expected signature with a leading 0", tt.algorithm)
		}
		sig.Blob = sig.Blob[1:]
		if err := pub.Verify(tt.data, sig); err != nil {
			t.Errorf("publicKey.Verify(%s): %v", tt.algorithm, err)
		}
	}
}

func TestParseRSAPrivateKey(t *testing.T) {
	key := testPrivateKeys["rsa"]

	rsa, ok := key.(*rsa.PrivateKey)
	if !ok {
		t.Fatalf("got %T, want *rsa.PrivateKey", rsa)
	}

	if err := rsa.Validate(); err != nil {
		t.Errorf("Validate: %v", err)
	}
}

func TestParseECPrivateKey(t *testing.T) {
	key := testPrivateKeys["ecdsa"]

	ecKey, ok := key.(*ecdsa.PrivateKey)
	if !ok {
		t.Fatalf("got %T, want *ecdsa.PrivateKey", ecKey)
	}

	if !validateECPublicKey(ecKey.Curve, ecKey.X, ecKey.Y) {
		t.Fatalf("public key does not validate.")
	}
}

func TestParseEncryptedPrivateKeysWithPassphrase(t *testing.T) {
	data := []byte("sign me")
	for _, tt := range testdata.PEMEncryptedKeys {
		t.Run(tt.Name, func(t *testing.T) {
			_, err := ParsePrivateKeyWithPassphrase(tt.PEMBytes, []byte("incorrect"))
			if err != x509.IncorrectPasswordError {
				t.Errorf("got %v want IncorrectPasswordError", err)
			}

			s, err := ParsePrivateKeyWithPassphrase(tt.PEMBytes, []byte(tt.EncryptionKey))
			if err != nil {
				t.Fatalf("ParsePrivateKeyWithPassphrase returned error: %s", err)
			}

			sig, err := s.Sign(rand.Reader, data)
			if err != nil {
				t.Fatalf("Signer.Sign: %v", err)
			}
			if err := s.PublicKey().Verify(data, sig); err != nil {
				t.Errorf("Verify failed: %v", err)
			}

			_, err = ParsePrivateKey(tt.PEMBytes)
			if err == nil {
				t.Fatalf("ParsePrivateKey succeeded, expected an error")
			}

			if err, ok := err.(*PassphraseMissingError); !ok {
				t.Errorf("got error %q, want PassphraseMissingError", err)
			} else if tt.IncludesPublicKey {
				if err.PublicKey == nil {
					t.Fatalf("expected PassphraseMissingError.PublicKey not to be nil")
				}
				got, want := err.PublicKey.Marshal(), s.PublicKey().Marshal()
				if !bytes.Equal(got, want) {
					t.Errorf("error field %q doesn't match signer public key %q", got, want)
				}
			}
		})
	}
}

func TestParseEncryptedPrivateKeysWithIncorrectPassphrase(t *testing.T) {
	pem := testdata.PEMEncryptedKeys[0].PEMBytes
	for i := 0; i < 4096; i++ {
		_, err := ParseRawPrivateKeyWithPassphrase(pem, []byte(fmt.Sprintf("%d", i)))
		if !errors.Is(err, x509.IncorrectPasswordError) {
			t.Fatalf("expected error: %v, got: %v", x509.IncorrectPasswordError, err)
		}
	}
}

func TestParseDSA(t *testing.T) {
	// We actually exercise the ParsePrivateKey codepath here, as opposed to
	// using the ParseRawPrivateKey+NewSignerFromKey path that testdata_test.go
	// uses.
	s, err := ParsePrivateKey(testdata.PEMBytes["dsa"])
	if err != nil {
		t.Fatalf("ParsePrivateKey returned error: %s", err)
	}

	data := []byte("sign me")
	sig, err := s.Sign(rand.Reader, data)
	if err != nil {
		t.Fatalf("dsa.Sign: %v", err)
	}

	if err := s.PublicKey().Verify(data, sig); err != nil {
		t.Errorf("Verify failed: %v", err)
	}
}

// Tests for authorized_keys parsing.

// getTestKey returns a public key, and its base64 encoding.
func getTestKey() (PublicKey, string) {
	k := testPublicKeys["rsa"]

	b := &bytes.Buffer{}
	e := base64.NewEncoder(base64.StdEncoding, b)
	e.Write(k.Marshal())
	e.Close()

	return k, b.String()
}

func TestMarshalParsePublicKey(t *testing.T) {
	pub, pubSerialized := getTestKey()
	line := fmt.Sprintf("%s %s user@host", pub.Type(), pubSerialized)

	authKeys := MarshalAuthorizedKey(pub)
	actualFields := strings.Fields(string(authKeys))
	if len(actualFields) == 0 {
		t.Fatalf("failed authKeys: %v", authKeys)
	}

	// drop the comment
	expectedFields := strings.Fields(line)[0:2]

	if !reflect.DeepEqual(actualFields, expectedFields) {
		t.Errorf("got %v, expected %v", actualFields, expectedFields)
	}

	actPub, _, _, _, err := ParseAuthorizedKey([]byte(line))
	if err != nil {
		t.Fatalf("cannot parse %v: %v", line, err)
	}
	if !reflect.DeepEqual(actPub, pub) {
		t.Errorf("got %v, expected %v", actPub, pub)
	}
}

func TestMarshalPrivateKey(t *testing.T) {
	tests := []struct {
		name string
	}{
		{"rsa-openssh-format"},
		{"ed25519"},
		{"p256-openssh-format"},
		{"p384-openssh-format"},
		{"p521-openssh-format"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			expected, ok := testPrivateKeys[tt.name]
			if !ok {
				t.Fatalf("cannot find key %s", tt.name)
			}

			block, err := MarshalPrivateKey(expected, "test@golang.org")
			if err != nil {
				t.Fatalf("cannot marshal %s: %v", tt.name, err)
			}

			key, err := ParseRawPrivateKey(pem.EncodeToMemory(block))
			if err != nil {
				t.Fatalf("cannot parse %s: %v", tt.name, err)
			}

			if !reflect.DeepEqual(expected, key) {
				t.Errorf("unexpected marshaled key %s", tt.name)
			}
		})
	}
}

func TestMarshalPrivateKeyWithPassphrase(t *testing.T) {
	tests := []struct {
		name string
	}{
		{"rsa-openssh-format"},
		{"ed25519"},
		{"p256-openssh-format"},
		{"p384-openssh-format"},
		{"p521-openssh-format"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			expected, ok := testPrivateKeys[tt.name]
			if !ok {
				t.Fatalf("cannot find key %s", tt.name)
			}

			block, err := MarshalPrivateKeyWithPassphrase(expected, "test@golang.org", []byte("test-passphrase"))
			if err != nil {
				t.Fatalf("cannot marshal %s: %v", tt.name, err)
			}

			key, err := ParseRawPrivateKeyWithPassphrase(pem.EncodeToMemory(block), []byte("test-passphrase"))
			if err != nil {
				t.Fatalf("cannot parse %s: %v", tt.name, err)
			}

			if !reflect.DeepEqual(expected, key) {
				t.Errorf("unexpected marshaled key %s", tt.name)
			}
		})
	}
}

type testAuthResult struct {
	pubKey   PublicKey
	options  []string
	comments string
	rest     string
	ok       bool
}

func testAuthorizedKeys(t *testing.T, authKeys []byte, expected []testAuthResult) {
	rest := authKeys
	var values []testAuthResult
	for len(rest) > 0 {
		var r testAuthResult
		var err error
		r.pubKey, r.comments, r.options, rest, err = ParseAuthorizedKey(rest)
		r.ok = (err == nil)
		t.Log(err)
		r.rest = string(rest)
		values = append(values, r)
	}

	if !reflect.DeepEqual(values, expected) {
		t.Errorf("got %#v, expected %#v", values, expected)
	}
}

func TestAuthorizedKeyBasic(t *testing.T) {
	pub, pubSerialized := getTestKey()
	line := "ssh-rsa " + pubSerialized + " user@host"
	testAuthorizedKeys(t, []byte(line),
		[]testAuthResult{
			{pub, nil, "user@host", "", true},
		})
}

func TestAuth(t *testing.T) {
	pub, pubSerialized := getTestKey()
	authWithOptions := []string{
		`# comments to ignore before any keys...`,
		``,
		`env="HOME=/home/root",no-port-forwarding ssh-rsa ` + pubSerialized + ` user@host`,
		`# comments to ignore, along with a blank line`,
		``,
		`env="HOME=/home/root2" ssh-rsa ` + pubSerialized + ` user2@host2`,
		``,
		`# more comments, plus a invalid entry`,
		`ssh-rsa data-that-will-not-parse user@host3`,
	}
	for _, eol := range []string{"\n", "\r\n"} {
		authOptions := strings.Join(authWithOptions, eol)
		rest2 := strings.Join(authWithOptions[3:], eol)
		rest3 := strings.Join(authWithOptions[6:], eol)
		testAuthorizedKeys(t, []byte(authOptions), []testAuthResult{
			{pub, []string{`env="HOME=/home/root"`, "no-port-forwarding"}, "user@host", rest2, true},
			{pub, []string{`env="HOME=/home/root2"`}, "user2@host2", rest3, true},
			{nil, nil, "", "", false},
		})
	}
}

func TestAuthWithQuotedSpaceInEnv(t *testing.T) {
	pub, pubSerialized := getTestKey()
	authWithQuotedSpaceInEnv := []byte(`env="HOME=/home/root dir",no-port-forwarding ssh-rsa ` + pubSerialized + ` user@host`)
	testAuthorizedKeys(t, []byte(authWithQuotedSpaceInEnv), []testAuthResult{
		{pub, []string{`env="HOME=/home/root dir"`, "no-port-forwarding"}, "user@host", "", true},
	})
}

func TestAuthWithQuotedCommaInEnv(t *testing.T) {
	pub, pubSerialized := getTestKey()
	authWithQuotedCommaInEnv := []byte(`env="HOME=/home/root,dir",no-port-forwarding ssh-rsa ` + pubSerialized + `   user@host`)
	testAuthorizedKeys(t, []byte(authWithQuotedCommaInEnv), []testAuthResult{
		{pub, []string{`env="HOME=/home/root,dir"`, "no-port-forwarding"}, "user@host", "", true},
	})
}

func TestAuthWithQuotedQuoteInEnv(t *testing.T) {
	pub, pubSerialized := getTestKey()
	authWithQuotedQuoteInEnv := []byte(`env="HOME=/home/\"root dir",no-port-forwarding` + "\t" + `ssh-rsa` + "\t" + pubSerialized + `   user@host`)
	authWithDoubleQuotedQuote := []byte(`no-port-forwarding,env="HOME=/home/ \"root dir\"" ssh-rsa ` + pubSerialized + "\t" + `user@host`)
	testAuthorizedKeys(t, []byte(authWithQuotedQuoteInEnv), []testAuthResult{
		{pub, []string{`env="HOME=/home/\"root dir"`, "no-port-forwarding"}, "user@host", "", true},
	})

	testAuthorizedKeys(t, []byte(authWithDoubleQuotedQuote), []testAuthResult{
		{pub, []string{"no-port-forwarding", `env="HOME=/home/ \"root dir\""`}, "user@host", "", true},
	})
}

func TestAuthWithInvalidSpace(t *testing.T) {
	_, pubSerialized := getTestKey()
	authWithInvalidSpace := []byte(`env="HOME=/home/root dir", no-port-forwarding ssh-rsa ` + pubSerialized + ` user@host
#more to follow but still no valid keys`)
	testAuthorizedKeys(t, []byte(authWithInvalidSpace), []testAuthResult{
		{nil, nil, "", "", false},
	})
}

func TestAuthWithMissingQuote(t *testing.T) {
	pub, pubSerialized := getTestKey()
	authWithMissingQuote := []byte(`env="HOME=/home/root,no-port-forwarding ssh-rsa ` + pubSerialized + ` user@host
env="HOME=/home/root",shared-control ssh-rsa ` + pubSerialized + ` user@host`)

	testAuthorizedKeys(t, []byte(authWithMissingQuote), []testAuthResult{
		{pub, []string{`env="HOME=/home/root"`, `shared-control`}, "user@host", "", true},
	})
}

func TestInvalidEntry(t *testing.T) {
	authInvalid := []byte(`ssh-rsa`)
	_, _, _, _, err := ParseAuthorizedKey(authInvalid)
	if err == nil {
		t.Errorf("got valid entry for %q", authInvalid)
	}
}

var knownHostsParseTests = []struct {
	input string
	err   string

	marker  string
	comment string
	hosts   []string
	rest    string
}{
	{
		"",
		"EOF",

		"", "", nil, "",
	},
	{
		"# Just a comment",
		"EOF",

		"", "", nil, "",
	},
	{
		"   \t   ",
		"EOF",

		"", "", nil, "",
	},
	{
		"localhost ssh-rsa {RSAPUB}",
		"",

		"", "", []string{"localhost"}, "",
	},
	{
		"localhost\tssh-rsa {RSAPUB}",
		"",

		"", "", []string{"localhost"}, "",
	},
	{
		"localhost\tssh-rsa {RSAPUB}\tcomment comment",
		"",

		"", "comment comment", []string{"localhost"}, "",
	},
	{
		"localhost\tssh-rsa {RSAPUB}\tcomment comment\n",
		"",

		"", "comment comment", []string{"localhost"}, "",
	},
	{
		"localhost\tssh-rsa {RSAPUB}\tcomment comment\r\n",
		"",

		"", "comment comment", []string{"localhost"}, "",
	},
	{
		"localhost\tssh-rsa {RSAPUB}\tcomment comment\r\nnext line",
		"",

		"", "comment comment", []string{"localhost"}, "next line",
	},
	{
		"localhost,[host2:123]\tssh-rsa {RSAPUB}\tcomment comment",
		"",

		"", "comment comment", []string{"localhost", "[host2:123]"}, "",
	},
	{
		"@marker \tlocalhost,[host2:123]\tssh-rsa {RSAPUB}",
		"",

		"marker", "", []string{"localhost", "[host2:123]"}, "",
	},
	{
		"@marker \tlocalhost,[host2:123]\tssh-rsa aabbccdd",
		"short read",

		"", "", nil, "",
	},
}

func TestKnownHostsParsing(t *testing.T) {
	rsaPub, rsaPubSerialized := getTestKey()

	for i, test := range knownHostsParseTests {
		var expectedKey PublicKey
		const rsaKeyToken = "{RSAPUB}"

		input := test.input
		if strings.Contains(input, rsaKeyToken) {
			expectedKey = rsaPub
			input = strings.Replace(test.input, rsaKeyToken, rsaPubSerialized, -1)
		}

		marker, hosts, pubKey, comment, rest, err := ParseKnownHosts([]byte(input))
		if err != nil {
			if len(test.err) == 0 {
				t.Errorf("#%d: unexpectedly failed with %q", i, err)
			} else if !strings.Contains(err.Error(), test.err) {
				t.Errorf("#%d: expected error containing %q, but got %q", i, test.err, err)
			}
			continue
		} else if len(test.err) != 0 {
			t.Errorf("#%d: succeeded but expected error including %q", i, test.err)
			continue
		}

		if !reflect.DeepEqual(expectedKey, pubKey) {
			t.Errorf("#%d: expected key %#v, but got %#v", i, expectedKey, pubKey)
		}

		if marker != test.marker {
			t.Errorf("#%d: expected marker %q, but got %q", i, test.marker, marker)
		}

		if comment != test.comment {
			t.Errorf("#%d: expected comment %q, but got %q", i, test.comment, comment)
		}

		if !reflect.DeepEqual(test.hosts, hosts) {
			t.Errorf("#%d: expected hosts %#v, but got %#v", i, test.hosts, hosts)
		}

		if rest := string(rest); rest != test.rest {
			t.Errorf("#%d: expected remaining input to be %q, but got %q", i, test.rest, rest)
		}
	}
}

func TestFingerprintLegacyMD5(t *testing.T) {
	pub, _ := getTestKey()
	fingerprint := FingerprintLegacyMD5(pub)
	want := "b7:ef:d3:d5:89:29:52:96:9f:df:47:41:4d:15:37:f4" // ssh-keygen -lf -E md5 rsa
	if fingerprint != want {
		t.Errorf("got fingerprint %q want %q", fingerprint, want)
	}
}

func TestFingerprintSHA256(t *testing.T) {
	pub, _ := getTestKey()
	fingerprint := FingerprintSHA256(pub)
	want := "SHA256:fi5+D7UmDZDE9Q2sAVvvlpcQSIakN4DERdINgXd2AnE" // ssh-keygen -lf rsa
	if fingerprint != want {
		t.Errorf("got fingerprint %q want %q", fingerprint, want)
	}
}

func TestInvalidKeys(t *testing.T) {
	keyTypes := []string{
		"RSA PRIVATE KEY",
		"PRIVATE KEY",
		"EC PRIVATE KEY",
		"DSA PRIVATE KEY",
		"OPENSSH PRIVATE KEY",
	}

	for _, keyType := range keyTypes {
		for _, dataLen := range []int{0, 1, 2, 5, 10, 20} {
			data := make([]byte, dataLen)
			if _, err := io.ReadFull(rand.Reader, data); err != nil {
				t.Fatal(err)
			}

			var buf bytes.Buffer
			pem.Encode(&buf, &pem.Block{
				Type:  keyType,
				Bytes: data,
			})

			// This test is just to ensure that the function
			// doesn't panic so the return value is ignored.
			ParseRawPrivateKey(buf.Bytes())
		}
	}
}

func TestSKKeys(t *testing.T) {
	for _, d := range testdata.SKData {
		pk, _, _, _, err := ParseAuthorizedKey(d.PubKey)
		if err != nil {
			t.Fatalf("parseAuthorizedKey returned error: %v", err)
		}

		sigBuf := make([]byte, hex.DecodedLen(len(d.HexSignature)))
		if _, err := hex.Decode(sigBuf, d.HexSignature); err != nil {
			t.Fatalf("hex.Decode() failed: %v", err)
		}

		dataBuf := make([]byte, hex.DecodedLen(len(d.HexData)))
		if _, err := hex.Decode(dataBuf, d.HexData); err != nil {
			t.Fatalf("hex.Decode() failed: %v", err)
		}

		sig, _, ok := parseSignature(sigBuf)
		if !ok {
			t.Fatalf("parseSignature(%v) failed", sigBuf)
		}

		// Test that good data and signature pass verification
		if err := pk.Verify(dataBuf, sig); err != nil {
			t.Errorf("%s: PublicKey.Verify(%v, %v) failed: %v", d.Name, dataBuf, sig, err)
		}

		// Invalid data being passed in
		invalidData := []byte("INVALID DATA")
		if err := pk.Verify(invalidData, sig); err == nil {
			t.Errorf("%s with invalid data: PublicKey.Verify(%v, %v) passed unexpectedly", d.Name, invalidData, sig)
		}

		// Change byte in blob to corrup signature
		sig.Blob[5] = byte('A')
		// Corrupted data being passed in
		if err := pk.Verify(dataBuf, sig); err == nil {
			t.Errorf("%s with corrupted signature: PublicKey.Verify(%v, %v) passed unexpectedly", d.Name, dataBuf, sig)
		}
	}
}

func TestNewSignerWithAlgos(t *testing.T) {
	algorithSigner, ok := testSigners["rsa"].(AlgorithmSigner)
	if !ok {
		t.Fatal("rsa test signer does not implement the AlgorithmSigner interface")
	}
	_, err := NewSignerWithAlgorithms(algorithSigner, nil)
	if err == nil {
		t.Error("signer with algos created with no algorithms")
	}

	_, err = NewSignerWithAlgorithms(algorithSigner, []string{KeyAlgoED25519})
	if err == nil {
		t.Error("signer with algos created with invalid algorithms")
	}

	_, err = NewSignerWithAlgorithms(algorithSigner, []string{CertAlgoRSASHA256v01})
	if err == nil {
		t.Error("signer with algos created with certificate algorithms")
	}

	mas, err := NewSignerWithAlgorithms(algorithSigner, []string{KeyAlgoRSASHA256, KeyAlgoRSASHA512})
	if err != nil {
		t.Errorf("unable to create signer with valid algorithms: %v", err)
	}

	_, err = NewSignerWithAlgorithms(mas, []string{KeyAlgoRSA})
	if err == nil {
		t.Error("signer with algos created with restricted algorithms")
	}
}

func TestCryptoPublicKey(t *testing.T) {
	for _, priv := range testSigners {
		p1 := priv.PublicKey()
		key, ok := p1.(CryptoPublicKey)
		if !ok {
			continue
		}
		p2, err := NewPublicKey(key.CryptoPublicKey())
		if err != nil {
			t.Fatalf("NewPublicKey(CryptoPublicKey) failed for %s, got: %v", p1.Type(), err)
		}
		if !reflect.DeepEqual(p1, p2) {
			t.Errorf("got %#v in NewPublicKey, want %#v", p2, p1)
		}
	}
	for _, d := range testdata.SKData {
		p1, _, _, _, err := ParseAuthorizedKey(d.PubKey)
		if err != nil {
			t.Fatalf("parseAuthorizedKey returned error: %v", err)
		}
		k1, ok := p1.(CryptoPublicKey)
		if !ok {
			t.Fatalf("%T does not implement CryptoPublicKey", p1)
		}

		var p2 PublicKey
		switch pub := k1.CryptoPublicKey().(type) {
		case *ecdsa.PublicKey:
			p2 = &skECDSAPublicKey{
				application: "ssh:",
				PublicKey:   *pub,
			}
		case ed25519.PublicKey:
			p2 = &skEd25519PublicKey{
				application: "ssh:",
				PublicKey:   pub,
			}
		default:
			t.Fatalf("unexpected type %T from CryptoPublicKey()", pub)
		}
		if !reflect.DeepEqual(p1, p2) {
			t.Errorf("got %#v, want %#v", p2, p1)
		}
	}
}
