/*
Deprecated.

Legacy types are defined below to aid in the migration of Tendermint consensus
parameters from use of the now deprecated x/params modules to a new dedicated
x/consensus module.

Application developers should ensure that they implement their upgrade handler
correctly such that app.ConsensusParamsKeeper.Set() is called with the values
returned by GetConsensusParams().

Example:

	baseAppLegacySS := app.ParamsKeeper.Subspace(baseapp.Paramspace).WithKeyTable(paramstypes.ConsensusParamsKeyTable())

	app.UpgradeKeeper.SetUpgradeHandler(
		UpgradeName,
		func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
			if cp := baseapp.GetConsensusParams(ctx, baseAppLegacySS); cp != nil {
				app.ConsensusParamsKeeper.Set(ctx, cp)
			} else {
				ctx.Logger().Info("warning: consensus parameters are undefined; skipping migration", "upgrade", UpgradeName)
			}

			return app.ModuleManager.RunMigrations(ctx, app.Configurator(), fromVM)
		},
	)

Developers can also bypass the use of the legacy Params subspace and set the
values to app.ConsensusParamsKeeper.Set() explicitly.

Note, for new chains this is not necessary as Tendermint's consensus parameters
will automatically be set for you in InitChain.
*/
package baseapp

import (
	"errors"
	"fmt"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const Paramspace = "baseapp"

var (
	ParamStoreKeyBlockParams     = []byte("BlockParams")
	ParamStoreKeyEvidenceParams  = []byte("EvidenceParams")
	ParamStoreKeyValidatorParams = []byte("ValidatorParams")
)

type LegacyParamStore interface {
	Get(ctx sdk.Context, key []byte, ptr interface{})
	Has(ctx sdk.Context, key []byte) bool
}

func ValidateBlockParams(i interface{}) error {
	v, ok := i.(tmproto.BlockParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.MaxBytes <= 0 {
		return fmt.Errorf("block maximum bytes must be positive: %d", v.MaxBytes)
	}

	if v.MaxGas < -1 {
		return fmt.Errorf("block maximum gas must be greater than or equal to -1: %d", v.MaxGas)
	}

	return nil
}

func ValidateEvidenceParams(i interface{}) error {
	v, ok := i.(tmproto.EvidenceParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.MaxAgeNumBlocks <= 0 {
		return fmt.Errorf("evidence maximum age in blocks must be positive: %d", v.MaxAgeNumBlocks)
	}

	if v.MaxAgeDuration <= 0 {
		return fmt.Errorf("evidence maximum age time duration must be positive: %v", v.MaxAgeDuration)
	}

	if v.MaxBytes < 0 {
		return fmt.Errorf("maximum evidence bytes must be non-negative: %v", v.MaxBytes)
	}

	return nil
}

func ValidateValidatorParams(i interface{}) error {
	v, ok := i.(tmproto.ValidatorParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if len(v.PubKeyTypes) == 0 {
		return errors.New("validator allowed pubkey types must not be empty")
	}

	return nil
}

func GetConsensusParams(ctx sdk.Context, paramStore LegacyParamStore) *tmproto.ConsensusParams {
	if paramStore == nil {
		return nil
	}

	cp := new(tmproto.ConsensusParams)

	if paramStore.Has(ctx, ParamStoreKeyBlockParams) {
		var bp tmproto.BlockParams

		paramStore.Get(ctx, ParamStoreKeyBlockParams, &bp)
		cp.Block = &bp
	}

	if paramStore.Has(ctx, ParamStoreKeyEvidenceParams) {
		var ep tmproto.EvidenceParams

		paramStore.Get(ctx, ParamStoreKeyEvidenceParams, &ep)
		cp.Evidence = &ep
	}

	if paramStore.Has(ctx, ParamStoreKeyValidatorParams) {
		var vp tmproto.ValidatorParams

		paramStore.Get(ctx, ParamStoreKeyValidatorParams, &vp)
		cp.Validator = &vp
	}

	return cp
}

func MigrateParams(ctx sdk.Context, lps LegacyParamStore, ps ParamStore) {
	if cp := GetConsensusParams(ctx, lps); cp != nil {
		ps.Set(ctx, cp)
	} else {
		ctx.Logger().Info("warning: consensus parameters are undefined; skipping migration")
	}
}
