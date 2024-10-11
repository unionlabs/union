package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ types.AuthorizationPolicy = DefaultAuthorizationPolicy{}

type DefaultAuthorizationPolicy struct{}

func (p DefaultAuthorizationPolicy) CanCreateCode(chainConfigs types.ChainAccessConfigs, actor sdk.AccAddress, contractConfig types.AccessConfig) bool {
	return chainConfigs.Upload.Allowed(actor) &&
		contractConfig.IsSubset(chainConfigs.Instantiate)
}

func (p DefaultAuthorizationPolicy) CanInstantiateContract(config types.AccessConfig, actor sdk.AccAddress) bool {
	return config.Allowed(actor)
}

func (p DefaultAuthorizationPolicy) CanModifyContract(admin, actor sdk.AccAddress) bool {
	return admin != nil && admin.Equals(actor)
}

func (p DefaultAuthorizationPolicy) CanModifyCodeAccessConfig(creator, actor sdk.AccAddress, isSubset bool) bool {
	return creator != nil && creator.Equals(actor) && isSubset
}

// SubMessageAuthorizationPolicy always returns the default policy
func (p DefaultAuthorizationPolicy) SubMessageAuthorizationPolicy(_ types.AuthorizationPolicyAction) types.AuthorizationPolicy {
	return p
}

var _ types.AuthorizationPolicy = GovAuthorizationPolicy{}

type GovAuthorizationPolicy struct {
	propagate map[types.AuthorizationPolicyAction]struct{}
}

// NewGovAuthorizationPolicy public constructor
func NewGovAuthorizationPolicy(actions ...types.AuthorizationPolicyAction) types.AuthorizationPolicy {
	propagate := make(map[types.AuthorizationPolicyAction]struct{}, len(actions))
	for _, a := range actions {
		propagate[a] = struct{}{}
	}
	return newGovAuthorizationPolicy(propagate)
}

// newGovAuthorizationPolicy internal constructor
func newGovAuthorizationPolicy(propagate map[types.AuthorizationPolicyAction]struct{}) types.AuthorizationPolicy {
	return GovAuthorizationPolicy{
		propagate: propagate,
	}
}

// CanCreateCode implements AuthorizationPolicy.CanCreateCode to allow gov actions. Always returns true.
func (p GovAuthorizationPolicy) CanCreateCode(types.ChainAccessConfigs, sdk.AccAddress, types.AccessConfig) bool {
	return true
}

func (p GovAuthorizationPolicy) CanInstantiateContract(types.AccessConfig, sdk.AccAddress) bool {
	return true
}

func (p GovAuthorizationPolicy) CanModifyContract(sdk.AccAddress, sdk.AccAddress) bool {
	return true
}

func (p GovAuthorizationPolicy) CanModifyCodeAccessConfig(sdk.AccAddress, sdk.AccAddress, bool) bool {
	return true
}

// SubMessageAuthorizationPolicy returns new policy with fine-grained gov permission for given action only
func (p GovAuthorizationPolicy) SubMessageAuthorizationPolicy(action types.AuthorizationPolicyAction) types.AuthorizationPolicy {
	defaultPolicy := DefaultAuthorizationPolicy{}
	if p.propagate != nil && len(p.propagate) != 0 {
		if _, ok := p.propagate[action]; ok {
			return NewPartialGovAuthorizationPolicy(defaultPolicy, action)
		}
	}
	return defaultPolicy
}

var _ types.AuthorizationPolicy = PartialGovAuthorizationPolicy{}

// PartialGovAuthorizationPolicy decorates the given default policy to add fine-grained gov permissions
// to the defined action
type PartialGovAuthorizationPolicy struct {
	action        types.AuthorizationPolicyAction
	defaultPolicy types.AuthorizationPolicy
}

// NewPartialGovAuthorizationPolicy constructor
func NewPartialGovAuthorizationPolicy(defaultPolicy types.AuthorizationPolicy, entrypoint types.AuthorizationPolicyAction) PartialGovAuthorizationPolicy {
	return PartialGovAuthorizationPolicy{action: entrypoint, defaultPolicy: defaultPolicy}
}

func (p PartialGovAuthorizationPolicy) CanCreateCode(chainConfigs types.ChainAccessConfigs, actor sdk.AccAddress, contractConfig types.AccessConfig) bool {
	return p.defaultPolicy.CanCreateCode(chainConfigs, actor, contractConfig)
}

func (p PartialGovAuthorizationPolicy) CanInstantiateContract(c types.AccessConfig, actor sdk.AccAddress) bool {
	if p.action == types.AuthZActionInstantiate {
		return true
	}
	return p.defaultPolicy.CanInstantiateContract(c, actor)
}

func (p PartialGovAuthorizationPolicy) CanModifyContract(admin, actor sdk.AccAddress) bool {
	if p.action == types.AuthZActionMigrateContract {
		return true
	}
	return p.defaultPolicy.CanModifyContract(admin, actor)
}

func (p PartialGovAuthorizationPolicy) CanModifyCodeAccessConfig(creator, actor sdk.AccAddress, isSubset bool) bool {
	return p.defaultPolicy.CanModifyCodeAccessConfig(creator, actor, isSubset)
}

// SubMessageAuthorizationPolicy always returns self
func (p PartialGovAuthorizationPolicy) SubMessageAuthorizationPolicy(_ types.AuthorizationPolicyAction) types.AuthorizationPolicy {
	return p
}
