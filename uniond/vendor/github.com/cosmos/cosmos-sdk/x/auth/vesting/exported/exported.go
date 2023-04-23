package exported

import (
	"time"

	"github.com/cosmos/cosmos-sdk/x/auth/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// VestingAccount defines an account type that vests coins via a vesting schedule.
type VestingAccount interface {
	types.AccountI

	// LockedCoins returns the set of coins that are not spendable (i.e. locked),
	// defined as the vesting coins that are not delegated.
	//
	// To get spendable coins of a vesting account, first the total balance must
	// be retrieved and the locked tokens can be subtracted from the total balance.
	// Note, the spendable balance can be negative.
	LockedCoins(blockTime time.Time) sdk.Coins

	// TrackDelegation performs internal vesting accounting necessary when
	// delegating from a vesting account. It accepts the current block time, the
	// delegation amount and balance of all coins whose denomination exists in
	// the account's original vesting balance.
	TrackDelegation(blockTime time.Time, balance, amount sdk.Coins)

	// TrackUndelegation performs internal vesting accounting necessary when a
	// vesting account performs an undelegation.
	TrackUndelegation(amount sdk.Coins)

	GetVestedCoins(blockTime time.Time) sdk.Coins
	GetVestingCoins(blockTime time.Time) sdk.Coins

	GetStartTime() int64
	GetEndTime() int64

	GetOriginalVesting() sdk.Coins
	GetDelegatedFree() sdk.Coins
	GetDelegatedVesting() sdk.Coins
}
