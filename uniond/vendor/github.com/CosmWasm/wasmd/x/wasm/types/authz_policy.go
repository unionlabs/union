package types

import (
	"github.com/cosmos/cosmos-sdk/types"
)

// ChainAccessConfigs chain settings
type ChainAccessConfigs struct {
	Upload      AccessConfig
	Instantiate AccessConfig
}

// NewChainAccessConfigs constructor
func NewChainAccessConfigs(upload, instantiate AccessConfig) ChainAccessConfigs {
	return ChainAccessConfigs{Upload: upload, Instantiate: instantiate}
}

type AuthorizationPolicyAction uint64

const (
	_ AuthorizationPolicyAction = iota
	AuthZActionInstantiate
	AuthZActionMigrateContract
)

// AuthorizationPolicy is an abstract authorization ruleset defined as an extension point that can be customized by
// chains
type AuthorizationPolicy interface {
	CanCreateCode(chainConfigs ChainAccessConfigs, actor types.AccAddress, contractConfig AccessConfig) bool
	CanInstantiateContract(c AccessConfig, actor types.AccAddress) bool
	CanModifyContract(admin, actor types.AccAddress) bool
	CanModifyCodeAccessConfig(creator, actor types.AccAddress, isSubset bool) bool
	// SubMessageAuthorizationPolicy returns authorization policy to be used for submessages. Must never be nil
	SubMessageAuthorizationPolicy(entrypoint AuthorizationPolicyAction) AuthorizationPolicy
}
