//go:build libsecp256k1_sdk
// +build libsecp256k1_sdk

package secp256k1

import (
	"testing"

	"github.com/magiconair/properties/assert"
	"github.com/stretchr/testify/require"
)

func TestPrivKeySecp256k1SignVerify(t *testing.T) {
	msg := []byte("A.1.2 ECC Key Pair Generation by Testing Candidates")
	priv := GenPrivKey()
	tests := []struct {
		name             string
		privKey          *PrivKey
		wantSignErr      bool
		wantVerifyPasses bool
	}{
		{name: "valid sign-verify round", privKey: priv, wantSignErr: false, wantVerifyPasses: true},
		{name: "invalid private key", privKey: &PrivKey{Key: []byte{}}, wantSignErr: true, wantVerifyPasses: false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.privKey.Sign(msg)
			if tt.wantSignErr {
				require.Error(t, err)
				t.Logf("Got error: %s", err)
				return
			}
			require.NoError(t, err)
			require.NotNil(t, got)

			pub := tt.privKey.PubKey()
			assert.Equal(t, tt.wantVerifyPasses, pub.VerifySignature(msg, got))
		})
	}
}
