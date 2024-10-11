package offchain

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
)

func Test_sign(t *testing.T) {
	k := keyring.NewInMemory(getCodec())

	ctx := client.Context{
		Keyring:      k,
		TxConfig:     newTestConfig(t),
		AddressCodec: address.NewBech32Codec("cosmos"),
	}

	type args struct {
		ctx      client.Context
		fromName string
		digest   string
	}
	tests := []struct {
		name string
		args args
	}{
		{
			name: "Sign",
			args: args{
				ctx:      ctx,
				fromName: "direct",
				digest:   "Hello world!",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := k.NewAccount(tt.args.fromName, mnemonic, tt.name, "m/44'/118'/0'/0/0", hd.Secp256k1)
			require.NoError(t, err)

			got, err := sign(tt.args.ctx, tt.args.fromName, tt.args.digest)
			require.NoError(t, err)
			require.NotNil(t, got)
		})
	}
}
