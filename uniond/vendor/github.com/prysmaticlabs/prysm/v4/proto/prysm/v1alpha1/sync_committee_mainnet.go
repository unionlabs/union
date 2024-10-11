//go:build !minimal

package eth

import (
	"github.com/prysmaticlabs/go-bitfield"
)

func NewSyncCommitteeAggregationBits() bitfield.Bitvector128 {
	return bitfield.NewBitvector128()
}

func ConvertToSyncContributionBitVector(b []byte) bitfield.Bitvector128 {
	return b
}
