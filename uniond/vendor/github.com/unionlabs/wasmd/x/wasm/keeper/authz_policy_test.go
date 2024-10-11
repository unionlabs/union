package keeper

import (
	"testing"

	"github.com/stretchr/testify/assert"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestDefaultAuthzPolicyCanCreateCode(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)
	specs := map[string]struct {
		chainConfigs     types.ChainAccessConfigs
		contractInstConf types.AccessConfig
		actor            sdk.AccAddress
		exp              bool
		panics           bool
	}{
		"upload nobody": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowNobody, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
			exp:              false,
		},
		"upload everybody": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
			exp:              true,
		},
		"upload any address - included": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessTypeAnyOfAddresses.With(otherAddress, myActorAddress), types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
			exp:              true,
		},
		"upload any address - not included": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessTypeAnyOfAddresses.With(otherAddress), types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
			exp:              false,
		},
		"contract config -  subtype": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowEverybody),
			contractInstConf: types.AccessTypeAnyOfAddresses.With(myActorAddress),
			exp:              true,
		},
		"contract config - not subtype": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowNobody),
			contractInstConf: types.AllowEverybody,
			exp:              false,
		},
		"upload undefined config - panics": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessConfig{}, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
			panics:           true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := DefaultAuthorizationPolicy{}
			if !spec.panics {
				got := policy.CanCreateCode(spec.chainConfigs, myActorAddress, spec.contractInstConf)
				assert.Equal(t, spec.exp, got)
				return
			}
			assert.Panics(t, func() {
				policy.CanCreateCode(spec.chainConfigs, myActorAddress, spec.contractInstConf)
			})
		})
	}
}

func TestDefaultAuthzPolicyCanInstantiateContract(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)
	specs := map[string]struct {
		config types.AccessConfig
		actor  sdk.AccAddress
		exp    bool
		panics bool
	}{
		"nobody": {
			config: types.AllowNobody,
			exp:    false,
		},
		"everybody": {
			config: types.AllowEverybody,
			exp:    true,
		},
		"any address - included": {
			config: types.AccessTypeAnyOfAddresses.With(otherAddress, myActorAddress),
			exp:    true,
		},
		"any address - not included": {
			config: types.AccessTypeAnyOfAddresses.With(otherAddress),
			exp:    false,
		},
		"undefined config - panics": {
			config: types.AccessConfig{},
			panics: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := DefaultAuthorizationPolicy{}
			if !spec.panics {
				got := policy.CanInstantiateContract(spec.config, myActorAddress)
				assert.Equal(t, spec.exp, got)
				return
			}
			assert.Panics(t, func() {
				policy.CanInstantiateContract(spec.config, myActorAddress)
			})
		})
	}
}

func TestDefaultAuthzPolicyCanModifyContract(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)

	specs := map[string]struct {
		admin sdk.AccAddress
		exp   bool
	}{
		"same as actor": {
			admin: myActorAddress,
			exp:   true,
		},
		"different admin": {
			admin: otherAddress,
			exp:   false,
		},
		"no admin": {
			exp: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := DefaultAuthorizationPolicy{}
			got := policy.CanModifyContract(spec.admin, myActorAddress)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestDefaultAuthzPolicyCanModifyCodeAccessConfig(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)

	specs := map[string]struct {
		admin  sdk.AccAddress
		subset bool
		exp    bool
	}{
		"same as actor - subset": {
			admin:  myActorAddress,
			subset: true,
			exp:    true,
		},
		"same as actor - not subset": {
			admin:  myActorAddress,
			subset: false,
			exp:    false,
		},
		"different admin": {
			admin: otherAddress,
			exp:   false,
		},
		"no admin": {
			exp: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := DefaultAuthorizationPolicy{}
			got := policy.CanModifyCodeAccessConfig(spec.admin, myActorAddress, spec.subset)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestDefaultAuthzPolicySubMessageAuthorizationPolicy(t *testing.T) {
	policy := DefaultAuthorizationPolicy{}
	for _, v := range []types.AuthorizationPolicyAction{types.AuthZActionInstantiate, types.AuthZActionMigrateContract} {
		got := policy.SubMessageAuthorizationPolicy(v)
		assert.Equal(t, policy, got)
	}
}

func TestGovAuthzPolicyCanCreateCode(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)
	specs := map[string]struct {
		chainConfigs     types.ChainAccessConfigs
		contractInstConf types.AccessConfig
		actor            sdk.AccAddress
	}{
		"upload nobody": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowNobody, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
		},
		"upload everybody": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
		},
		"upload any address - included": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessTypeAnyOfAddresses.With(otherAddress, myActorAddress), types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
		},
		"upload any address - not included": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessTypeAnyOfAddresses.With(otherAddress), types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
		},
		"contract config -  subtype": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowEverybody),
			contractInstConf: types.AccessTypeAnyOfAddresses.With(myActorAddress),
		},
		"contract config - not subtype": {
			chainConfigs:     types.NewChainAccessConfigs(types.AllowEverybody, types.AllowNobody),
			contractInstConf: types.AllowEverybody,
		},
		"upload undefined config - not panics": {
			chainConfigs:     types.NewChainAccessConfigs(types.AccessConfig{}, types.AllowEverybody),
			contractInstConf: types.AllowEverybody,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := GovAuthorizationPolicy{}
			got := policy.CanCreateCode(spec.chainConfigs, myActorAddress, spec.contractInstConf)
			assert.True(t, got)
		})
	}
}

func TestGovAuthzPolicyCanInstantiateContract(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)
	specs := map[string]struct {
		config types.AccessConfig
		actor  sdk.AccAddress
	}{
		"nobody": {
			config: types.AllowNobody,
		},
		"everybody": {
			config: types.AllowEverybody,
		},
		"any address - included": {
			config: types.AccessTypeAnyOfAddresses.With(otherAddress, myActorAddress),
		},
		"any address - not included": {
			config: types.AccessTypeAnyOfAddresses.With(otherAddress),
		},
		"undefined config - panics": {
			config: types.AccessConfig{},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := GovAuthorizationPolicy{}
			got := policy.CanInstantiateContract(spec.config, myActorAddress)
			assert.True(t, got)
		})
	}
}

func TestGovAuthzPolicyCanModifyContract(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)

	specs := map[string]struct {
		admin sdk.AccAddress
	}{
		"same as actor": {
			admin: myActorAddress,
		},
		"different admin": {
			admin: otherAddress,
		},
		"no admin": {},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := GovAuthorizationPolicy{}
			got := policy.CanModifyContract(spec.admin, myActorAddress)
			assert.True(t, got)
		})
	}
}

func TestGovAuthzPolicyCanModifyCodeAccessConfig(t *testing.T) {
	myActorAddress := RandomAccountAddress(t)
	otherAddress := RandomAccountAddress(t)

	specs := map[string]struct {
		admin  sdk.AccAddress
		subset bool
	}{
		"same as actor - subset": {
			admin:  myActorAddress,
			subset: true,
		},
		"same as actor - not subset": {
			admin:  myActorAddress,
			subset: false,
		},
		"different admin": {
			admin: otherAddress,
		},
		"no admin": {},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := newGovAuthorizationPolicy(nil)
			got := policy.CanModifyCodeAccessConfig(spec.admin, myActorAddress, spec.subset)
			assert.True(t, got)
		})
	}
}

func TestGovAuthorizationPolicySubMessageAuthorizationPolicy(t *testing.T) {
	specs := map[string]struct {
		propagate  map[types.AuthorizationPolicyAction]struct{}
		entrypoint types.AuthorizationPolicyAction
		exp        types.AuthorizationPolicy
	}{
		"non propagating": {
			exp: DefaultAuthorizationPolicy{},
		},
		"propagating with matching action": {
			propagate: map[types.AuthorizationPolicyAction]struct{}{
				types.AuthZActionMigrateContract: {},
			},
			entrypoint: types.AuthZActionMigrateContract,
			exp:        NewPartialGovAuthorizationPolicy(DefaultAuthorizationPolicy{}, types.AuthZActionMigrateContract),
		},
		"propagating for non matching action": {
			propagate: map[types.AuthorizationPolicyAction]struct{}{
				types.AuthZActionMigrateContract: {},
			},
			entrypoint: types.AuthZActionInstantiate,
			exp:        DefaultAuthorizationPolicy{},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			got := newGovAuthorizationPolicy(spec.propagate).SubMessageAuthorizationPolicy(spec.entrypoint)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestPartialGovAuthorizationPolicyCanInstantiateContract(t *testing.T) {
	specs := map[string]struct {
		allowedAction types.AuthorizationPolicyAction
		exp           bool
	}{
		"instantiation granted": {
			allowedAction: types.AuthZActionInstantiate,
			exp:           true,
		},
		"decorated policy when instantiation not granted ": {
			allowedAction: types.AuthZActionMigrateContract,
			exp:           false,
		},
		"decorated policy when nothing set": {
			exp: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := NewPartialGovAuthorizationPolicy(AlwaysRejectTestAuthZPolicy{}, spec.allowedAction)
			got := policy.CanInstantiateContract(types.AccessConfig{}, nil)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestPartialGovAuthorizationPolicyCanModifyContract(t *testing.T) {
	specs := map[string]struct {
		allowedAction types.AuthorizationPolicyAction
		exp           bool
	}{
		"migration granted": {
			allowedAction: types.AuthZActionMigrateContract,
			exp:           true,
		},
		"decorated policy when migration not granted ": {
			allowedAction: types.AuthZActionInstantiate,
			exp:           false,
		},
		"decorated policy when nothing set": {
			exp: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			policy := NewPartialGovAuthorizationPolicy(AlwaysRejectTestAuthZPolicy{}, spec.allowedAction)
			got := policy.CanModifyContract(nil, nil)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestPartialGovAuthorizationPolicyDelegatedOnly(t *testing.T) {
	for _, v := range []types.AuthorizationPolicy{AlwaysRejectTestAuthZPolicy{}, NewGovAuthorizationPolicy()} {
		policy := NewPartialGovAuthorizationPolicy(v, types.AuthZActionInstantiate)

		got := policy.CanCreateCode(types.ChainAccessConfigs{}, nil, types.AccessConfig{})
		exp := v.CanCreateCode(types.ChainAccessConfigs{}, nil, types.AccessConfig{})
		assert.Equal(t, exp, got)

		got = policy.CanModifyCodeAccessConfig(nil, nil, false)
		exp = v.CanModifyCodeAccessConfig(nil, nil, false)
		assert.Equal(t, exp, got)
	}
}

func TestPartialGovAuthorizationPolicySubMessageAuthorizationPolicy(t *testing.T) {
	policy := NewPartialGovAuthorizationPolicy(DefaultAuthorizationPolicy{}, types.AuthZActionInstantiate)
	for _, v := range []types.AuthorizationPolicyAction{types.AuthZActionInstantiate, types.AuthZActionMigrateContract} {
		got := policy.SubMessageAuthorizationPolicy(v)
		assert.Equal(t, policy, got)
	}
}

var _ types.AuthorizationPolicy = AlwaysRejectTestAuthZPolicy{}

type AlwaysRejectTestAuthZPolicy struct{}

func (a AlwaysRejectTestAuthZPolicy) CanCreateCode(chainConfigs types.ChainAccessConfigs, actor sdk.AccAddress, contractConfig types.AccessConfig) bool {
	return false
}

func (a AlwaysRejectTestAuthZPolicy) CanInstantiateContract(c types.AccessConfig, actor sdk.AccAddress) bool {
	return false
}

func (a AlwaysRejectTestAuthZPolicy) CanModifyContract(admin, actor sdk.AccAddress) bool {
	return false
}

func (a AlwaysRejectTestAuthZPolicy) CanModifyCodeAccessConfig(creator, actor sdk.AccAddress, isSubset bool) bool {
	return false
}

func (a AlwaysRejectTestAuthZPolicy) SubMessageAuthorizationPolicy(entrypoint types.AuthorizationPolicyAction) types.AuthorizationPolicy {
	return a
}
