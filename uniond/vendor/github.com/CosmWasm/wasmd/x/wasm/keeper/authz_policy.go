package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// ChainAccessConfigs chain settings
type ChainAccessConfigs struct {
	Upload      types.AccessConfig
	Instantiate types.AccessConfig
}

// NewChainAccessConfigs constructor
func NewChainAccessConfigs(upload types.AccessConfig, instantiate types.AccessConfig) ChainAccessConfigs {
	return ChainAccessConfigs{Upload: upload, Instantiate: instantiate}
}

type AuthorizationPolicy interface {
	CanCreateCode(chainConfigs ChainAccessConfigs, actor sdk.AccAddress, contractConfig types.AccessConfig) bool
	CanInstantiateContract(c types.AccessConfig, actor sdk.AccAddress) bool
	CanModifyContract(admin, actor sdk.AccAddress) bool
	CanModifyCodeAccessConfig(creator, actor sdk.AccAddress, isSubset bool) bool
}

type DefaultAuthorizationPolicy struct{}

func (p DefaultAuthorizationPolicy) CanCreateCode(chainConfigs ChainAccessConfigs, actor sdk.AccAddress, contractConfig types.AccessConfig) bool {
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

type GovAuthorizationPolicy struct{}

// CanCreateCode implements AuthorizationPolicy.CanCreateCode to allow gov actions. Always returns true.
func (p GovAuthorizationPolicy) CanCreateCode(ChainAccessConfigs, sdk.AccAddress, types.AccessConfig) bool {
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
