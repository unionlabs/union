package validator

import (
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

type Status int8

const (
	PendingInitialized Status = iota
	PendingQueued
	ActiveOngoing
	ActiveExiting
	ActiveSlashed
	ExitedUnslashed
	ExitedSlashed
	WithdrawalPossible
	WithdrawalDone
	Active
	Pending
	Exited
	Withdrawal
)

func (s Status) String() string {
	switch s {
	case PendingInitialized:
		return "pending_initialized"
	case PendingQueued:
		return "pending_queued"
	case ActiveOngoing:
		return "active_ongoing"
	case ActiveExiting:
		return "active_exiting"
	case ActiveSlashed:
		return "active_slashed"
	case ExitedUnslashed:
		return "exited_unslashed"
	case ExitedSlashed:
		return "exited_slashed"
	case WithdrawalPossible:
		return "withdrawal_possible"
	case WithdrawalDone:
		return "withdrawal_done"
	case Active:
		return "active"
	case Pending:
		return "pending"
	case Exited:
		return "exited"
	case Withdrawal:
		return "withdrawal"
	default:
		return "unknown"
	}
}

func StatusFromString(s string) (bool, Status) {
	switch s {
	case "pending_initialized":
		return true, PendingInitialized
	case "pending_queued":
		return true, PendingQueued
	case "active_ongoing":
		return true, ActiveOngoing
	case "active_exiting":
		return true, ActiveExiting
	case "active_slashed":
		return true, ActiveSlashed
	case "exited_unslashed":
		return true, ExitedUnslashed
	case "exited_slashed":
		return true, ExitedSlashed
	case "withdrawal_possible":
		return true, WithdrawalPossible
	case "withdrawal_done":
		return true, WithdrawalDone
	case "active":
		return true, Active
	case "pending":
		return true, Pending
	case "exited":
		return true, Exited
	case "withdrawal":
		return true, Withdrawal
	default:
		return false, -1
	}
}

type SyncCommitteeSubscription struct {
	ValidatorIndex       primitives.ValidatorIndex
	SyncCommitteeIndices []uint64
	UntilEpoch           primitives.Epoch
}

type BeaconCommitteeSubscription struct {
	ValidatorIndex   primitives.ValidatorIndex
	CommitteeIndex   primitives.CommitteeIndex
	CommitteesAtSlot uint64
	Slot             primitives.Slot
	IsAggregator     bool
}
