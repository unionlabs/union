package v2_test

import (
	"encoding/binary"
	"testing"

	"github.com/stretchr/testify/require"

	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	v2 "github.com/cosmos/cosmos-sdk/x/group/migrations/v2"
)

func TestMigrateGenState(t *testing.T) {
	tests := []struct {
		name     string
		oldState *authtypes.GenesisState
		newState *authtypes.GenesisState
	}{
		{
			name: "group policy accounts are replaced by base accounts",
			oldState: authtypes.NewGenesisState(authtypes.DefaultParams(), authtypes.GenesisAccounts{
				&authtypes.ModuleAccount{
					BaseAccount: &authtypes.BaseAccount{
						Address:       "cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl",
						AccountNumber: 3,
					},
					Name:        "distribution",
					Permissions: []string{},
				},
				&authtypes.ModuleAccount{
					BaseAccount: &authtypes.BaseAccount{
						Address:       "cosmos1q32tjg5qm3n9fj8wjgpd7gl98prefntrckjkyvh8tntp7q33zj0s5tkjrk",
						AccountNumber: 8,
					},
					Name:        "cosmos1q32tjg5qm3n9fj8wjgpd7gl98prefntrckjkyvh8tntp7q33zj0s5tkjrk",
					Permissions: []string{},
				},
			}),
			newState: authtypes.NewGenesisState(authtypes.DefaultParams(), authtypes.GenesisAccounts{
				&authtypes.ModuleAccount{
					BaseAccount: &authtypes.BaseAccount{
						Address:       "cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl",
						AccountNumber: 3,
					},
					Name:        "distribution",
					Permissions: []string{},
				},
				func() *authtypes.BaseAccount {
					baseAccount := &authtypes.BaseAccount{
						Address:       "cosmos1q32tjg5qm3n9fj8wjgpd7gl98prefntrckjkyvh8tntp7q33zj0s5tkjrk",
						AccountNumber: 8,
					}

					k := make([]byte, 8)
					binary.BigEndian.PutUint64(k, 0)
					c, err := authtypes.NewModuleCredential(group.ModuleName, []byte{v2.GroupPolicyTablePrefix}, k)
					if err != nil {
						panic(err)
					}
					err = baseAccount.SetPubKey(c)
					if err != nil {
						panic(err)
					}

					return baseAccount
				}(),
			},
			),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			require.Error(t, authtypes.ValidateGenesis(*tc.oldState))
			actualState := v2.MigrateGenState(tc.oldState)
			require.Equal(t, tc.newState, actualState)
			require.NoError(t, authtypes.ValidateGenesis(*actualState))
		})
	}
}
