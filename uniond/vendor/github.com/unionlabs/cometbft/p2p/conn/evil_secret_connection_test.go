package conn

import (
	"bufio"
	"bytes"
	"errors"
	"io"
	"testing"

	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/oasisprotocol/curve25519-voi/primitives/merlin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/chacha20poly1305"

	tmp2p "github.com/cometbft/cometbft/api/cometbft/p2p/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/libs/protoio"
)

type buffer struct {
	next bytes.Buffer
}

func (b *buffer) Read(data []byte) (n int, err error) {
	return b.next.Read(data)
}

func (b *buffer) Write(data []byte) (n int, err error) {
	return b.next.Write(data)
}

func (b *buffer) Bytes() []byte {
	return b.next.Bytes()
}

func (*buffer) Close() error {
	return nil
}

type evilConn struct {
	secretConn *SecretConnection
	buffer     *buffer

	locEphPub  *[32]byte
	locEphPriv *[32]byte
	remEphPub  *[32]byte
	privKey    crypto.PrivKey

	readStep   int
	writeStep  int
	readOffset int

	shareEphKey        bool
	badEphKey          bool
	shareAuthSignature bool
	badAuthSignature   bool
}

func newEvilConn(shareEphKey, badEphKey, shareAuthSignature, badAuthSignature bool) *evilConn {
	privKey := ed25519.GenPrivKey()
	locEphPub, locEphPriv := genEphKeys()
	var rep [32]byte
	c := &evilConn{
		locEphPub:  locEphPub,
		locEphPriv: locEphPriv,
		remEphPub:  &rep,
		privKey:    privKey,

		shareEphKey:        shareEphKey,
		badEphKey:          badEphKey,
		shareAuthSignature: shareAuthSignature,
		badAuthSignature:   badAuthSignature,
	}

	return c
}

func (c *evilConn) Read(data []byte) (n int, err error) {
	if !c.shareEphKey {
		return 0, io.EOF
	}

	switch c.readStep {
	case 0:
		if !c.badEphKey {
			lc := *c.locEphPub
			bz, err := protoio.MarshalDelimited(&gogotypes.BytesValue{Value: lc[:]})
			if err != nil {
				panic(err)
			}
			copy(data, bz[c.readOffset:])
			n = len(data)
		} else {
			bz, err := protoio.MarshalDelimited(&gogotypes.BytesValue{Value: []byte("drop users;")})
			if err != nil {
				panic(err)
			}
			copy(data, bz)
			n = len(data)
		}
		c.readOffset += n

		if n >= 32 {
			c.readOffset = 0
			c.readStep = 1
			if !c.shareAuthSignature {
				c.readStep = 2
			}
		}

		return n, nil
	case 1:
		signature := c.signChallenge()
		if !c.badAuthSignature {
			pkpb, err := cryptoenc.PubKeyToProto(c.privKey.PubKey())
			if err != nil {
				panic(err)
			}
			bz, err := protoio.MarshalDelimited(&tmp2p.AuthSigMessage{PubKey: pkpb, Sig: signature})
			if err != nil {
				panic(err)
			}
			n, err = c.secretConn.Write(bz)
			if err != nil {
				panic(err)
			}
			if c.readOffset > len(c.buffer.Bytes()) {
				return len(data), nil
			}
			copy(data, c.buffer.Bytes()[c.readOffset:])
		} else {
			bz, err := protoio.MarshalDelimited(&gogotypes.BytesValue{Value: []byte("select * from users;")})
			if err != nil {
				panic(err)
			}
			n, err = c.secretConn.Write(bz)
			if err != nil {
				panic(err)
			}
			if c.readOffset > len(c.buffer.Bytes()) {
				return len(data), nil
			}
			copy(data, c.buffer.Bytes())
		}
		c.readOffset += len(data)
		return n, nil
	default:
		return 0, io.EOF
	}
}

func (c *evilConn) Write(data []byte) (n int, err error) {
	switch c.writeStep {
	case 0:
		var (
			bytes     gogotypes.BytesValue
			remEphPub [32]byte
		)
		err := protoio.UnmarshalDelimited(data, &bytes)
		if err != nil {
			panic(err)
		}
		copy(remEphPub[:], bytes.Value)
		c.remEphPub = &remEphPub
		c.writeStep = 1
		if !c.shareAuthSignature {
			c.writeStep = 2
		}
		return len(data), nil
	case 1:
		// Signature is not needed, therefore skipped.
		return len(data), nil
	default:
		return 0, io.EOF
	}
}

func (*evilConn) Close() error {
	return nil
}

func (c *evilConn) signChallenge() []byte {
	// Sort by lexical order.
	loEphPub, hiEphPub := sort32(c.locEphPub, c.remEphPub)

	transcript := merlin.NewTranscript("TENDERMINT_SECRET_CONNECTION_TRANSCRIPT_HASH")

	transcript.AppendMessage(labelEphemeralLowerPublicKey, loEphPub[:])
	transcript.AppendMessage(labelEphemeralUpperPublicKey, hiEphPub[:])

	// Check if the local ephemeral public key was the least, lexicographically
	// sorted.
	locIsLeast := bytes.Equal(c.locEphPub[:], loEphPub[:])

	// Compute common diffie hellman secret using X25519.
	dhSecret, err := computeDHSecret(c.remEphPub, c.locEphPriv)
	if err != nil {
		panic(err)
	}

	transcript.AppendMessage(labelDHSecret, dhSecret[:])

	// Generate the secret used for receiving, sending, challenge via HKDF-SHA2
	// on the transcript state (which itself also uses HKDF-SHA2 to derive a key
	// from the dhSecret).
	recvSecret, sendSecret := deriveSecrets(dhSecret, locIsLeast)

	const challengeSize = 32
	var challenge [challengeSize]byte
	transcript.ExtractBytes(challenge[:], labelSecretConnectionMac)

	sendAead, err := chacha20poly1305.New(sendSecret[:])
	if err != nil {
		panic(errors.New("invalid send SecretConnection Key"))
	}
	recvAead, err := chacha20poly1305.New(recvSecret[:])
	if err != nil {
		panic(errors.New("invalid receive SecretConnection Key"))
	}

	b := &buffer{}
	c.secretConn = &SecretConnection{
		conn:            b,
		connWriter:      bufio.NewWriterSize(b, defaultWriteBufferSize),
		connReader:      b,
		recvBuffer:      nil,
		recvNonce:       new([aeadNonceSize]byte),
		sendNonce:       new([aeadNonceSize]byte),
		recvAead:        recvAead,
		sendAead:        sendAead,
		recvFrame:       make([]byte, totalFrameSize),
		recvSealedFrame: make([]byte, totalFrameSize+aeadSizeOverhead),
		sendFrame:       make([]byte, totalFrameSize),
		sendSealedFrame: make([]byte, totalFrameSize+aeadSizeOverhead),
	}
	c.buffer = b

	// Sign the challenge bytes for authentication.
	locSignature, err := signChallenge(&challenge, c.privKey)
	if err != nil {
		panic(err)
	}

	return locSignature
}

// TestMakeSecretConnection creates an evil connection and tests that
// MakeSecretConnection errors at different stages.
func TestMakeSecretConnection(t *testing.T) {
	testCases := []struct {
		name   string
		conn   *evilConn
		errMsg string
	}{
		{"refuse to share ethimeral key", newEvilConn(false, false, false, false), "EOF"},
		{"share bad ethimeral key", newEvilConn(true, true, false, false), "wrong wireType"},
		{"refuse to share auth signature", newEvilConn(true, false, false, false), "EOF"},
		{"share bad auth signature", newEvilConn(true, false, true, true), "failed to decrypt SecretConnection"},
		// fails with the introduction of changes PR #3419
		// {"all good", newEvilConn(true, false, true, false), ""},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			privKey := ed25519.GenPrivKey()
			_, err := MakeSecretConnection(tc.conn, privKey)
			if tc.errMsg != "" {
				if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
					assert.Contains(t, err.Error(), tc.errMsg)
				}
			} else {
				require.NoError(t, err)
			}
		})
	}
}
