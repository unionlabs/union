package types

import (
	"context"
	"time"

	st "cosmossdk.io/api/cosmos/staking/v1beta1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/math"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// StakingKeeper defines the staking module interface contract needed by the
// evidence module.
type StakingKeeper interface {
	ConsensusAddressCodec() address.Codec
	ValidatorByConsAddr(context.Context, sdk.ConsAddress) (sdk.ValidatorI, error)
}

// SlashingKeeper defines the slashing module interface contract needed by the
// evidence module.
type SlashingKeeper interface {
	GetPubkey(context.Context, cryptotypes.Address) (cryptotypes.PubKey, error)
	IsTombstoned(context.Context, sdk.ConsAddress) bool
	HasValidatorSigningInfo(context.Context, sdk.ConsAddress) bool
	Tombstone(context.Context, sdk.ConsAddress) error
	Slash(context.Context, sdk.ConsAddress, math.LegacyDec, int64, int64) error
	SlashWithInfractionReason(context.Context, sdk.ConsAddress, math.LegacyDec, int64, int64, st.Infraction) error
	SlashFractionDoubleSign(context.Context) (math.LegacyDec, error)
	Jail(context.Context, sdk.ConsAddress) error
	JailUntil(context.Context, sdk.ConsAddress, time.Time) error
}

// AccountKeeper define the account keeper interface contracted needed by the evidence module
type AccountKeeper interface {
	SetAccount(ctx context.Context, acc sdk.AccountI)
}

type ConsensusKeeper interface {
	EvidenceParams(context.Context) (maxAge int64, maxAgeDuration time.Duration, maxBytes uint64, err error)
}
