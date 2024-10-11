// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"bytes"
	"crypto/rand"
	"errors"
	"fmt"
	"net"
	"strings"
	"testing"
)

func TestClientVersion(t *testing.T) {
	for _, tt := range []struct {
		name      string
		version   string
		multiLine string
		wantErr   bool
	}{
		{
			name:    "default version",
			version: packageVersion,
		},
		{
			name:    "custom version",
			version: "SSH-2.0-CustomClientVersionString",
		},
		{
			name:      "good multi line version",
			version:   packageVersion,
			multiLine: strings.Repeat("ignored\r\n", 20),
		},
		{
			name:      "bad multi line version",
			version:   packageVersion,
			multiLine: "bad multi line version",
			wantErr:   true,
		},
		{
			name:      "long multi line version",
			version:   packageVersion,
			multiLine: strings.Repeat("long multi line version\r\n", 50)[:256],
			wantErr:   true,
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			c1, c2, err := netPipe()
			if err != nil {
				t.Fatalf("netPipe: %v", err)
			}
			defer c1.Close()
			defer c2.Close()
			go func() {
				if tt.multiLine != "" {
					c1.Write([]byte(tt.multiLine))
				}
				NewClientConn(c1, "", &ClientConfig{
					ClientVersion:   tt.version,
					HostKeyCallback: InsecureIgnoreHostKey(),
				})
				c1.Close()
			}()
			conf := &ServerConfig{NoClientAuth: true}
			conf.AddHostKey(testSigners["rsa"])
			conn, _, _, err := NewServerConn(c2, conf)
			if err == nil == tt.wantErr {
				t.Fatalf("got err %v; wantErr %t", err, tt.wantErr)
			}
			if tt.wantErr {
				// Don't verify the version on an expected error.
				return
			}
			if got := string(conn.ClientVersion()); got != tt.version {
				t.Fatalf("got %q; want %q", got, tt.version)
			}
		})
	}
}

func TestHostKeyCheck(t *testing.T) {
	for _, tt := range []struct {
		name      string
		wantError string
		key       PublicKey
	}{
		{"no callback", "must specify HostKeyCallback", nil},
		{"correct key", "", testSigners["rsa"].PublicKey()},
		{"mismatch", "mismatch", testSigners["ecdsa"].PublicKey()},
	} {
		c1, c2, err := netPipe()
		if err != nil {
			t.Fatalf("netPipe: %v", err)
		}
		defer c1.Close()
		defer c2.Close()
		serverConf := &ServerConfig{
			NoClientAuth: true,
		}
		serverConf.AddHostKey(testSigners["rsa"])

		go NewServerConn(c1, serverConf)
		clientConf := ClientConfig{
			User: "user",
		}
		if tt.key != nil {
			clientConf.HostKeyCallback = FixedHostKey(tt.key)
		}

		_, _, _, err = NewClientConn(c2, "", &clientConf)
		if err != nil {
			if tt.wantError == "" || !strings.Contains(err.Error(), tt.wantError) {
				t.Errorf("%s: got error %q, missing %q", tt.name, err.Error(), tt.wantError)
			}
		} else if tt.wantError != "" {
			t.Errorf("%s: succeeded, but want error string %q", tt.name, tt.wantError)
		}
	}
}

func TestVerifyHostKeySignature(t *testing.T) {
	for _, tt := range []struct {
		key        string
		signAlgo   string
		verifyAlgo string
		wantError  string
	}{
		{"rsa", KeyAlgoRSA, KeyAlgoRSA, ""},
		{"rsa", KeyAlgoRSASHA256, KeyAlgoRSASHA256, ""},
		{"rsa", KeyAlgoRSA, KeyAlgoRSASHA512, `ssh: invalid signature algorithm "ssh-rsa", expected "rsa-sha2-512"`},
		{"ed25519", KeyAlgoED25519, KeyAlgoED25519, ""},
	} {
		key := testSigners[tt.key].PublicKey()
		s, ok := testSigners[tt.key].(AlgorithmSigner)
		if !ok {
			t.Fatalf("needed an AlgorithmSigner")
		}
		sig, err := s.SignWithAlgorithm(rand.Reader, []byte("test"), tt.signAlgo)
		if err != nil {
			t.Fatalf("couldn't sign: %q", err)
		}

		b := bytes.Buffer{}
		writeString(&b, []byte(sig.Format))
		writeString(&b, sig.Blob)

		result := kexResult{Signature: b.Bytes(), H: []byte("test")}

		err = verifyHostKeySignature(key, tt.verifyAlgo, &result)
		if err != nil {
			if tt.wantError == "" || !strings.Contains(err.Error(), tt.wantError) {
				t.Errorf("got error %q, expecting %q", err.Error(), tt.wantError)
			}
		} else if tt.wantError != "" {
			t.Errorf("succeeded, but want error string %q", tt.wantError)
		}
	}
}

func TestBannerCallback(t *testing.T) {
	c1, c2, err := netPipe()
	if err != nil {
		t.Fatalf("netPipe: %v", err)
	}
	defer c1.Close()
	defer c2.Close()

	serverConf := &ServerConfig{
		PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
			return &Permissions{}, nil
		},
		BannerCallback: func(conn ConnMetadata) string {
			return "Hello World"
		},
	}
	serverConf.AddHostKey(testSigners["rsa"])
	go NewServerConn(c1, serverConf)

	var receivedBanner string
	var bannerCount int
	clientConf := ClientConfig{
		Auth: []AuthMethod{
			Password("123"),
		},
		User:            "user",
		HostKeyCallback: InsecureIgnoreHostKey(),
		BannerCallback: func(message string) error {
			bannerCount++
			receivedBanner = message
			return nil
		},
	}

	_, _, _, err = NewClientConn(c2, "", &clientConf)
	if err != nil {
		t.Fatal(err)
	}

	if bannerCount != 1 {
		t.Errorf("got %d banners; want 1", bannerCount)
	}

	expected := "Hello World"
	if receivedBanner != expected {
		t.Fatalf("got %s; want %s", receivedBanner, expected)
	}
}

func TestNewClientConn(t *testing.T) {
	errHostKeyMismatch := errors.New("host key mismatch")

	for _, tt := range []struct {
		name                    string
		user                    string
		simulateHostKeyMismatch HostKeyCallback
	}{
		{
			name: "good user field for ConnMetadata",
			user: "testuser",
		},
		{
			name: "empty user field for ConnMetadata",
			user: "",
		},
		{
			name: "host key mismatch",
			user: "testuser",
			simulateHostKeyMismatch: func(hostname string, remote net.Addr, key PublicKey) error {
				return fmt.Errorf("%w: %s", errHostKeyMismatch, bytes.TrimSpace(MarshalAuthorizedKey(key)))
			},
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			c1, c2, err := netPipe()
			if err != nil {
				t.Fatalf("netPipe: %v", err)
			}
			defer c1.Close()
			defer c2.Close()

			serverConf := &ServerConfig{
				PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
					return &Permissions{}, nil
				},
			}
			serverConf.AddHostKey(testSigners["rsa"])
			go NewServerConn(c1, serverConf)

			clientConf := &ClientConfig{
				User: tt.user,
				Auth: []AuthMethod{
					Password("testpw"),
				},
				HostKeyCallback: InsecureIgnoreHostKey(),
			}

			if tt.simulateHostKeyMismatch != nil {
				clientConf.HostKeyCallback = tt.simulateHostKeyMismatch
			}

			clientConn, _, _, err := NewClientConn(c2, "", clientConf)
			if err != nil {
				if tt.simulateHostKeyMismatch != nil && errors.Is(err, errHostKeyMismatch) {
					return
				}
				t.Fatal(err)
			}

			if userGot := clientConn.User(); userGot != tt.user {
				t.Errorf("got user %q; want user %q", userGot, tt.user)
			}
		})
	}
}

func TestUnsupportedAlgorithm(t *testing.T) {
	for _, tt := range []struct {
		name      string
		config    Config
		wantError string
	}{
		{
			"unsupported KEX",
			Config{
				KeyExchanges: []string{"unsupported"},
			},
			"no common algorithm",
		},
		{
			"unsupported and supported KEXs",
			Config{
				KeyExchanges: []string{"unsupported", kexAlgoCurve25519SHA256},
			},
			"",
		},
		{
			"unsupported cipher",
			Config{
				Ciphers: []string{"unsupported"},
			},
			"no common algorithm",
		},
		{
			"unsupported and supported ciphers",
			Config{
				Ciphers: []string{"unsupported", chacha20Poly1305ID},
			},
			"",
		},
		{
			"unsupported MAC",
			Config{
				MACs: []string{"unsupported"},
				// MAC is used for non AAED ciphers.
				Ciphers: []string{"aes256-ctr"},
			},
			"no common algorithm",
		},
		{
			"unsupported and supported MACs",
			Config{
				MACs: []string{"unsupported", "hmac-sha2-256-etm@openssh.com"},
				// MAC is used for non AAED ciphers.
				Ciphers: []string{"aes256-ctr"},
			},
			"",
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			c1, c2, err := netPipe()
			if err != nil {
				t.Fatalf("netPipe: %v", err)
			}
			defer c1.Close()
			defer c2.Close()

			serverConf := &ServerConfig{
				Config: tt.config,
				PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
					return &Permissions{}, nil
				},
			}
			serverConf.AddHostKey(testSigners["rsa"])
			go NewServerConn(c1, serverConf)

			clientConf := &ClientConfig{
				User:   "testuser",
				Config: tt.config,
				Auth: []AuthMethod{
					Password("testpw"),
				},
				HostKeyCallback: InsecureIgnoreHostKey(),
			}
			_, _, _, err = NewClientConn(c2, "", clientConf)
			if err != nil {
				if tt.wantError == "" || !strings.Contains(err.Error(), tt.wantError) {
					t.Errorf("%s: got error %q, missing %q", tt.name, err.Error(), tt.wantError)
				}
			} else if tt.wantError != "" {
				t.Errorf("%s: succeeded, but want error string %q", tt.name, tt.wantError)
			}
		})
	}
}
