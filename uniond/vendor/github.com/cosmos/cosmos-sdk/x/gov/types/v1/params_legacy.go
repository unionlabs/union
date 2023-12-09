package v1

import (
	"errors"
	"fmt"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

// Parameter store key
var (
	ParamStoreKeyDepositParams = []byte("depositparams")
	ParamStoreKeyVotingParams  = []byte("votingparams")
	ParamStoreKeyTallyParams   = []byte("tallyparams")
)

// Deprecated: ParamKeyTable - Key declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable(
		paramtypes.NewParamSetPair(ParamStoreKeyDepositParams, DepositParams{}, validateDepositParams),
		paramtypes.NewParamSetPair(ParamStoreKeyVotingParams, VotingParams{}, validateVotingParams),
		paramtypes.NewParamSetPair(ParamStoreKeyTallyParams, TallyParams{}, validateTallyParams),
	)
}

func validateDepositParams(i interface{}) error {
	v, ok := i.(DepositParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if !sdk.Coins(v.MinDeposit).IsValid() {
		return fmt.Errorf("invalid minimum deposit: %s", v.MinDeposit)
	}
	if v.MaxDepositPeriod == nil || v.MaxDepositPeriod.Seconds() <= 0 {
		return fmt.Errorf("maximum deposit period must be positive: %d", v.MaxDepositPeriod)
	}

	return nil
}

func validateTallyParams(i interface{}) error {
	v, ok := i.(TallyParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	quorum, err := math.LegacyNewDecFromStr(v.Quorum)
	if err != nil {
		return fmt.Errorf("invalid quorum string: %w", err)
	}
	if quorum.IsNegative() {
		return fmt.Errorf("quorom cannot be negative: %s", quorum)
	}
	if quorum.GT(math.LegacyOneDec()) {
		return fmt.Errorf("quorom too large: %s", v)
	}

	threshold, err := math.LegacyNewDecFromStr(v.Threshold)
	if err != nil {
		return fmt.Errorf("invalid threshold string: %w", err)
	}
	if !threshold.IsPositive() {
		return fmt.Errorf("vote threshold must be positive: %s", threshold)
	}
	if threshold.GT(math.LegacyOneDec()) {
		return fmt.Errorf("vote threshold too large: %s", v)
	}

	vetoThreshold, err := math.LegacyNewDecFromStr(v.VetoThreshold)
	if err != nil {
		return fmt.Errorf("invalid vetoThreshold string: %w", err)
	}
	if !vetoThreshold.IsPositive() {
		return fmt.Errorf("veto threshold must be positive: %s", vetoThreshold)
	}
	if vetoThreshold.GT(math.LegacyOneDec()) {
		return fmt.Errorf("veto threshold too large: %s", v)
	}

	return nil
}

func validateVotingParams(i interface{}) error {
	v, ok := i.(VotingParams)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.VotingPeriod == nil {
		return errors.New("voting period must not be nil")
	}

	if v.VotingPeriod.Seconds() <= 0 {
		return fmt.Errorf("voting period must be positive: %s", v.VotingPeriod)
	}

	return nil
}
