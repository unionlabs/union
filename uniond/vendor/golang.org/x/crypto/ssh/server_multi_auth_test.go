// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"bytes"
	"errors"
	"fmt"
	"strings"
	"testing"
)

func doClientServerAuth(t *testing.T, serverConfig *ServerConfig, clientConfig *ClientConfig) ([]error, error) {
	c1, c2, err := netPipe()
	if err != nil {
		t.Fatalf("netPipe: %v", err)
	}
	defer c1.Close()
	defer c2.Close()

	var serverAuthErrors []error

	serverConfig.AddHostKey(testSigners["rsa"])
	serverConfig.AuthLogCallback = func(conn ConnMetadata, method string, err error) {
		serverAuthErrors = append(serverAuthErrors, err)
	}
	go newServer(c1, serverConfig)
	c, _, _, err := NewClientConn(c2, "", clientConfig)
	if err == nil {
		c.Close()
	}
	return serverAuthErrors, err
}

func TestMultiStepAuth(t *testing.T) {
	// This user can login with password, public key or public key + password.
	username := "testuser"
	// This user can login with public key + password only.
	usernameSecondFactor := "testuser_second_factor"
	errPwdAuthFailed := errors.New("password auth failed")
	errWrongSequence := errors.New("wrong sequence")

	serverConfig := &ServerConfig{
		PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
			if conn.User() == usernameSecondFactor {
				return nil, errWrongSequence
			}
			if conn.User() == username && string(password) == clientPassword {
				return nil, nil
			}
			return nil, errPwdAuthFailed
		},
		PublicKeyCallback: func(conn ConnMetadata, key PublicKey) (*Permissions, error) {
			if bytes.Equal(key.Marshal(), testPublicKeys["rsa"].Marshal()) {
				if conn.User() == usernameSecondFactor {
					return nil, &PartialSuccessError{
						Next: ServerAuthCallbacks{
							PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
								if string(password) == clientPassword {
									return nil, nil
								}
								return nil, errPwdAuthFailed
							},
						},
					}
				}
				return nil, nil
			}
			return nil, fmt.Errorf("pubkey for %q not acceptable", conn.User())
		},
	}

	clientConfig := &ClientConfig{
		User: usernameSecondFactor,
		Auth: []AuthMethod{
			PublicKeys(testSigners["rsa"]),
			Password(clientPassword),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	serverAuthErrors, err := doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("client login error: %s", err)
	}

	// The error sequence is:
	// - no auth passed yet
	// - partial success
	// - nil
	if len(serverAuthErrors) != 3 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[1].(*PartialSuccessError); !ok {
		t.Fatalf("expected partial success error, got: %v", serverAuthErrors[1])
	}
	// Now test a wrong sequence.
	clientConfig.Auth = []AuthMethod{
		Password(clientPassword),
		PublicKeys(testSigners["rsa"]),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("client login with wrong sequence must fail")
	}
	// The error sequence is:
	// - no auth passed yet
	// - wrong sequence
	// - partial success
	if len(serverAuthErrors) != 3 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if serverAuthErrors[1] != errWrongSequence {
		t.Fatal("server not returned wrong sequence")
	}
	if _, ok := serverAuthErrors[2].(*PartialSuccessError); !ok {
		t.Fatalf("expected partial success error, got: %v", serverAuthErrors[2])
	}
	// Now test using a correct sequence but a wrong password before the right
	// one.
	n := 0
	passwords := []string{"WRONG", "WRONG", clientPassword}
	clientConfig.Auth = []AuthMethod{
		PublicKeys(testSigners["rsa"]),
		RetryableAuthMethod(PasswordCallback(func() (string, error) {
			p := passwords[n]
			n++
			return p, nil
		}), 3),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("client login error: %s", err)
	}
	// The error sequence is:
	// - no auth passed yet
	// - partial success
	// - wrong password
	// - wrong password
	// - nil
	if len(serverAuthErrors) != 5 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[1].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}
	if serverAuthErrors[2] != errPwdAuthFailed {
		t.Fatal("server not returned password authentication failed")
	}
	if serverAuthErrors[3] != errPwdAuthFailed {
		t.Fatal("server not returned password authentication failed")
	}
	// Only password authentication should fail.
	clientConfig.Auth = []AuthMethod{
		Password(clientPassword),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("client login with password only must fail")
	}
	// The error sequence is:
	// - no auth passed yet
	// - wrong sequence
	if len(serverAuthErrors) != 2 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if serverAuthErrors[1] != errWrongSequence {
		t.Fatal("server not returned wrong sequence")
	}

	// Only public key authentication should fail.
	clientConfig.Auth = []AuthMethod{
		PublicKeys(testSigners["rsa"]),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("client login with public key only must fail")
	}
	// The error sequence is:
	// - no auth passed yet
	// - partial success
	if len(serverAuthErrors) != 2 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[1].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}

	// Public key and wrong password.
	clientConfig.Auth = []AuthMethod{
		PublicKeys(testSigners["rsa"]),
		Password("WRONG"),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("client login with wrong password after public key must fail")
	}
	// The error sequence is:
	// - no auth passed yet
	// - partial success
	// - password auth failed
	if len(serverAuthErrors) != 3 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[1].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}
	if serverAuthErrors[2] != errPwdAuthFailed {
		t.Fatal("server not returned password authentication failed")
	}

	// Public key, public key again and then correct password. Public key
	// authentication is attempted only once because the partial success error
	// returns only "password" as the allowed authentication method.
	clientConfig.Auth = []AuthMethod{
		PublicKeys(testSigners["rsa"]),
		PublicKeys(testSigners["rsa"]),
		Password(clientPassword),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("client login error: %s", err)
	}
	// The error sequence is:
	// - no auth passed yet
	// - partial success
	// - nil
	if len(serverAuthErrors) != 3 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[1].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}

	// The unrestricted username can do anything
	clientConfig = &ClientConfig{
		User: username,
		Auth: []AuthMethod{
			PublicKeys(testSigners["rsa"]),
			Password(clientPassword),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	_, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("unrestricted client login error: %s", err)
	}

	clientConfig = &ClientConfig{
		User: username,
		Auth: []AuthMethod{
			PublicKeys(testSigners["rsa"]),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	_, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("unrestricted client login error: %s", err)
	}

	clientConfig = &ClientConfig{
		User: username,
		Auth: []AuthMethod{
			Password(clientPassword),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	_, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("unrestricted client login error: %s", err)
	}
}

func TestDynamicAuthCallbacks(t *testing.T) {
	user1 := "user1"
	user2 := "user2"
	errInvalidCredentials := errors.New("invalid credentials")

	serverConfig := &ServerConfig{
		NoClientAuth: true,
		NoClientAuthCallback: func(conn ConnMetadata) (*Permissions, error) {
			switch conn.User() {
			case user1:
				return nil, &PartialSuccessError{
					Next: ServerAuthCallbacks{
						PasswordCallback: func(conn ConnMetadata, password []byte) (*Permissions, error) {
							if conn.User() == user1 && string(password) == clientPassword {
								return nil, nil
							}
							return nil, errInvalidCredentials
						},
					},
				}
			case user2:
				return nil, &PartialSuccessError{
					Next: ServerAuthCallbacks{
						PublicKeyCallback: func(conn ConnMetadata, key PublicKey) (*Permissions, error) {
							if bytes.Equal(key.Marshal(), testPublicKeys["rsa"].Marshal()) {
								if conn.User() == user2 {
									return nil, nil
								}
							}
							return nil, errInvalidCredentials
						},
					},
				}
			default:
				return nil, errInvalidCredentials
			}
		},
	}

	clientConfig := &ClientConfig{
		User: user1,
		Auth: []AuthMethod{
			Password(clientPassword),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	serverAuthErrors, err := doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("client login error: %s", err)
	}
	// The error sequence is:
	// - partial success
	// - nil
	if len(serverAuthErrors) != 2 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[0].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}

	clientConfig = &ClientConfig{
		User: user2,
		Auth: []AuthMethod{
			PublicKeys(testSigners["rsa"]),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err != nil {
		t.Fatalf("client login error: %s", err)
	}
	// The error sequence is:
	// - partial success
	// - nil
	if len(serverAuthErrors) != 2 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[0].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}

	// user1 cannot login with public key
	clientConfig = &ClientConfig{
		User: user1,
		Auth: []AuthMethod{
			PublicKeys(testSigners["rsa"]),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("user1 login with public key must fail")
	}
	if !strings.Contains(err.Error(), "no supported methods remain") {
		t.Errorf("got %v, expected 'no supported methods remain'", err)
	}
	if len(serverAuthErrors) != 1 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[0].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}
	// user2 cannot login with password
	clientConfig = &ClientConfig{
		User: user2,
		Auth: []AuthMethod{
			Password(clientPassword),
		},
		HostKeyCallback: InsecureIgnoreHostKey(),
	}

	serverAuthErrors, err = doClientServerAuth(t, serverConfig, clientConfig)
	if err == nil {
		t.Fatal("user2 login with password must fail")
	}
	if !strings.Contains(err.Error(), "no supported methods remain") {
		t.Errorf("got %v, expected 'no supported methods remain'", err)
	}
	if len(serverAuthErrors) != 1 {
		t.Fatalf("unexpected number of server auth errors: %v, errors: %+v", len(serverAuthErrors), serverAuthErrors)
	}
	if _, ok := serverAuthErrors[0].(*PartialSuccessError); !ok {
		t.Fatal("server not returned partial success")
	}
}
