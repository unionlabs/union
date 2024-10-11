package precompute

import "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"

// Validator stores the pre computation of individual validator's attesting records these records
// consist of attestation votes, block inclusion record. Pre computing and storing such record
// is essential for process epoch optimizations.
type Validator struct {
	// IsSlashed is true if the validator has been slashed.
	IsSlashed bool
	// IsWithdrawableCurrentEpoch is true if the validator can withdraw current epoch.
	IsWithdrawableCurrentEpoch bool
	// IsActiveCurrentEpoch is true if the validator was active current epoch.
	IsActiveCurrentEpoch bool
	// IsActivePrevEpoch is true if the validator was active prev epoch.
	IsActivePrevEpoch bool
	// IsCurrentEpochAttester is true if the validator attested current epoch.
	IsCurrentEpochAttester bool
	// IsCurrentEpochTargetAttester is true if the validator attested current epoch target.
	IsCurrentEpochTargetAttester bool
	// IsPrevEpochAttester is true if the validator attested previous epoch.
	IsPrevEpochAttester bool
	// IsPrevEpochSourceAttester is true if the validator attested to source previous epoch. [Only for Altair]
	IsPrevEpochSourceAttester bool
	// IsPrevEpochTargetAttester is true if the validator attested previous epoch target.
	IsPrevEpochTargetAttester bool
	// IsHeadAttester is true if the validator attested head.
	IsPrevEpochHeadAttester bool

	// CurrentEpochEffectiveBalance is how much effective balance this validator has current epoch.
	CurrentEpochEffectiveBalance uint64
	// InclusionSlot is the slot of when the attestation gets included in the chain.
	InclusionSlot primitives.Slot
	// InclusionDistance is the distance between the assigned slot and this validator's attestation was included in block.
	InclusionDistance primitives.Slot
	// ProposerIndex is the index of proposer at slot where this validator's attestation was included.
	ProposerIndex primitives.ValidatorIndex
	// BeforeEpochTransitionBalance is the validator balance prior to epoch transition.
	BeforeEpochTransitionBalance uint64
	// AfterEpochTransitionBalance is the validator balance after epoch transition.
	AfterEpochTransitionBalance uint64

	// InactivityScore of the validator. [New in Altair]
	InactivityScore uint64
}

// Balance stores the pre computation of the total participated balances for a given epoch
// Pre computing and storing such record is essential for process epoch optimizations.
type Balance struct {
	// ActiveCurrentEpoch is the total effective balance of all active validators during current epoch.
	ActiveCurrentEpoch uint64
	// ActivePrevEpoch is the total effective balance of all active validators during prev epoch.
	ActivePrevEpoch uint64
	// CurrentEpochAttested is the total effective balance of all validators who attested during current epoch.
	CurrentEpochAttested uint64
	// CurrentEpochTargetAttested is the total effective balance of all validators who attested
	// for epoch boundary block during current epoch.
	CurrentEpochTargetAttested uint64
	// PrevEpochAttested is the total effective balance of all validators who attested during prev epoch.
	PrevEpochAttested uint64
	// PrevEpochTargetAttested is the total effective balance of all validators who attested
	// for epoch boundary block during prev epoch.
	PrevEpochTargetAttested uint64
	// PrevEpochHeadAttested is the total effective balance of all validators who attested
	// correctly for head block during prev epoch.
	PrevEpochHeadAttested uint64
}
