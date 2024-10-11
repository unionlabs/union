package common

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
)

func TestQueryDelegationRewardsAddrValidation(t *testing.T) {
	cdcOpts := codectestutil.CodecOptions{}
	clientCtx := client.Context{}.
		WithLegacyAmino(legacy.Cdc).
		WithAddressCodec(cdcOpts.GetAddressCodec()).
		WithValidatorAddressCodec(cdcOpts.GetValidatorCodec())

	type args struct {
		delAddr string
		valAddr string
	}

	tests := []struct {
		name    string
		args    args
		want    []byte
		wantErr bool
	}{
		{"invalid delegator address", args{"invalid", ""}, nil, true},
		{"empty delegator address", args{"", ""}, nil, true},
		{"invalid validator address", args{"cosmos1zxcsu7l5qxs53lvp0fqgd09a9r2g6kqrk2cdpa", "invalid"}, nil, true},
		{"empty validator address", args{"cosmos1zxcsu7l5qxs53lvp0fqgd09a9r2g6kqrk2cdpa", ""}, nil, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, _, err := QueryDelegationRewards(clientCtx, tt.args.delAddr, tt.args.valAddr)
			require.True(t, err != nil, tt.wantErr)
		})
	}
}
