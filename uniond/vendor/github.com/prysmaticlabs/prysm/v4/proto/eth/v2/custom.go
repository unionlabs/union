package eth

import (
	"bytes"
	"math/bits"
)

const (
	NextSyncCommitteeIndex = uint64(55)
	FinalizedRootIndex     = uint64(105)
)

func (x *SyncCommittee) Equals(other *SyncCommittee) bool {
	if len(x.Pubkeys) != len(other.Pubkeys) {
		return false
	}
	for i := range x.Pubkeys {
		if !bytes.Equal(x.Pubkeys[i], other.Pubkeys[i]) {
			return false
		}
	}
	return bytes.Equal(x.AggregatePubkey, other.AggregatePubkey)
}

func FloorLog2(x uint64) int {
	return bits.Len64(x - 1)
}

func isEmptyWithLength(bb [][]byte, length uint64) bool {
	if len(bb) == 0 {
		return true
	}
	l := FloorLog2(length)
	if len(bb) != l {
		return false
	}
	for _, b := range bb {
		if !bytes.Equal(b, []byte{}) {
			return false
		}
	}
	return true
}

func (x *LightClientUpdate) IsSyncCommiteeUpdate() bool {
	return !isEmptyWithLength(x.GetNextSyncCommitteeBranch(), NextSyncCommitteeIndex)
}

func (x *LightClientUpdate) IsFinalityUpdate() bool {
	return !isEmptyWithLength(x.GetFinalityBranch(), FinalizedRootIndex)
}
