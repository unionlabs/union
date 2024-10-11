package tx

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

func TestFactoryPrepare(t *testing.T) {
	t.Parallel()

	factory := Factory{}
	clientCtx := client.Context{}

	output, err := factory.Prepare(clientCtx.WithOffline(true))
	require.NoError(t, err)
	require.Equal(t, output, factory)

	factory = Factory{}.WithAccountRetriever(client.MockAccountRetriever{ReturnAccNum: 10, ReturnAccSeq: 1}).WithAccountNumber(5)
	output, err = factory.Prepare(clientCtx.WithFrom("foo"))
	require.NoError(t, err)
	require.NotEqual(t, output, factory)
	require.Equal(t, output.AccountNumber(), uint64(5))
	require.Equal(t, output.Sequence(), uint64(1))

	factory = Factory{}.WithAccountRetriever(client.MockAccountRetriever{ReturnAccNum: 10, ReturnAccSeq: 1})
	output, err = factory.Prepare(clientCtx.WithFrom("foo"))
	require.NoError(t, err)
	require.NotEqual(t, output, factory)
	require.Equal(t, output.AccountNumber(), uint64(10))
	require.Equal(t, output.Sequence(), uint64(1))
}

func TestFactory_getSimPKType(t *testing.T) {
	// setup keyring
	registry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)
	k := keyring.NewInMemory(codec.NewProtoCodec(registry))

	tests := []struct {
		name     string
		fromName string
		genKey   func(fromName string, k keyring.Keyring) error
		wantType types.PubKey
	}{
		{
			name:     "simple key",
			fromName: "testKey",
			genKey: func(fromName string, k keyring.Keyring) error {
				_, err := k.NewAccount(fromName, testdata.TestMnemonic, "", "", hd.Secp256k1)
				return err
			},
			wantType: (*secp256k1.PubKey)(nil),
		},
		{
			name:     "multisig key",
			fromName: "multiKey",
			genKey: func(fromName string, k keyring.Keyring) error {
				pk := multisig.NewLegacyAminoPubKey(1, []types.PubKey{&multisig.LegacyAminoPubKey{}})
				_, err := k.SaveMultisig(fromName, pk)
				return err
			},
			wantType: (*multisig.LegacyAminoPubKey)(nil),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.genKey(tt.fromName, k)
			require.NoError(t, err)
			f := Factory{
				keybase:            k,
				fromName:           tt.fromName,
				simulateAndExecute: true,
			}
			got, err := f.getSimPK()
			require.NoError(t, err)
			require.IsType(t, tt.wantType, got)
		})
	}
}

func TestFactory_getSimSignatureData(t *testing.T) {
	tests := []struct {
		name     string
		pk       types.PubKey
		wantType any
	}{
		{
			name:     "simple pubkey",
			pk:       &secp256k1.PubKey{},
			wantType: (*signing.SingleSignatureData)(nil),
		},
		{
			name:     "multisig pubkey",
			pk:       &multisig.LegacyAminoPubKey{},
			wantType: (*signing.MultiSignatureData)(nil),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := Factory{}.getSimSignatureData(tt.pk)
			require.IsType(t, tt.wantType, got)
		})
	}
}
