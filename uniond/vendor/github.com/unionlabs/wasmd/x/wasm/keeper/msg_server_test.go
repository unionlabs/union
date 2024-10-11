package keeper

import (
	"testing"

	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/assert"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	storemetrics "cosmossdk.io/store/metrics"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestSelectAuthorizationPolicy(t *testing.T) {
	myGovAuthority := RandomAccountAddress(t)
	m := msgServer{keeper: &Keeper{
		propagateGovAuthorization: map[types.AuthorizationPolicyAction]struct{}{
			types.AuthZActionMigrateContract: {},
			types.AuthZActionInstantiate:     {},
		},
		authority: myGovAuthority.String(),
	}}

	ms := store.NewCommitMultiStore(dbm.NewMemDB(), log.NewTestLogger(t), storemetrics.NewNoOpMetrics())
	ctx := sdk.NewContext(ms, tmproto.Header{}, false, log.NewNopLogger())

	specs := map[string]struct {
		ctx   sdk.Context
		actor sdk.AccAddress
		exp   types.AuthorizationPolicy
	}{
		"always gov policy for gov authority sender": {
			ctx:   types.WithSubMsgAuthzPolicy(ctx, NewPartialGovAuthorizationPolicy(nil, types.AuthZActionMigrateContract)),
			actor: myGovAuthority,
			exp:   NewGovAuthorizationPolicy(types.AuthZActionMigrateContract, types.AuthZActionInstantiate),
		},
		"pick from context when set": {
			ctx:   types.WithSubMsgAuthzPolicy(ctx, NewPartialGovAuthorizationPolicy(nil, types.AuthZActionMigrateContract)),
			actor: RandomAccountAddress(t),
			exp:   NewPartialGovAuthorizationPolicy(nil, types.AuthZActionMigrateContract),
		},
		"fallback to default policy": {
			ctx:   ctx,
			actor: RandomAccountAddress(t),
			exp:   DefaultAuthorizationPolicy{},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			got := m.selectAuthorizationPolicy(spec.ctx, spec.actor.String())
			assert.Equal(t, spec.exp, got)
		})
	}
}
