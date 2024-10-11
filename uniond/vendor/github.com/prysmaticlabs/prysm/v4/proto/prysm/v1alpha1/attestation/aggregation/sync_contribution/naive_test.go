package sync_contribution

import (
	"fmt"
	"sort"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation"
	aggtesting "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestAggregateAttestations_aggregate(t *testing.T) {
	tests := []struct {
		a1   *ethpb.SyncCommitteeContribution
		a2   *ethpb.SyncCommitteeContribution
		want *ethpb.SyncCommitteeContribution
	}{
		{
			a1:   &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x02}, Signature: bls.NewAggregateSignature().Marshal()},
			a2:   &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x01}, Signature: bls.NewAggregateSignature().Marshal()},
			want: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x03}},
		},
		{
			a1:   &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x01}, Signature: bls.NewAggregateSignature().Marshal()},
			a2:   &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x02}, Signature: bls.NewAggregateSignature().Marshal()},
			want: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x03}},
		},
	}
	for _, tt := range tests {
		got, err := aggregate(tt.a1, tt.a2)
		require.NoError(t, err)
		require.DeepSSZEqual(t, tt.want.AggregationBits, got.AggregationBits)
	}
}

func TestAggregateAttestations_aggregate_OverlapFails(t *testing.T) {
	tests := []struct {
		a1 *ethpb.SyncCommitteeContribution
		a2 *ethpb.SyncCommitteeContribution
	}{
		{
			a1: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x1F}},
			a2: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x11}},
		},
		{
			a1: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0xFF, 0x85}},
			a2: &ethpb.SyncCommitteeContribution{AggregationBits: bitfield.Bitvector128{0x13, 0x8F}},
		},
	}
	for _, tt := range tests {
		_, err := aggregate(tt.a1, tt.a2)
		require.ErrorContains(t, aggregation.ErrBitsOverlap.Error(), err)
	}
}

func TestAggregateAttestations_Aggregate(t *testing.T) {
	tests := []struct {
		name   string
		inputs []bitfield.Bitvector128
		want   []bitfield.Bitvector128
	}{
		{
			name:   "empty list",
			inputs: []bitfield.Bitvector128{},
			want:   []bitfield.Bitvector128{},
		},
		{
			name: "single attestation",
			inputs: []bitfield.Bitvector128{
				{0b00000010},
			},
			want: []bitfield.Bitvector128{
				{0b00000010},
			},
		},
		{
			name: "two attestations with no overlap",
			inputs: []bitfield.Bitvector128{
				{0b00000001},
				{0b00000010},
			},
			want: []bitfield.Bitvector128{
				{0b00000011},
			},
		},
		{
			name: "two attestations with overlap",
			inputs: []bitfield.Bitvector128{
				{0b00000101},
				{0b00000110},
			},
			want: []bitfield.Bitvector128{
				{0b00000101},
				{0b00000110},
			},
		},
		{
			name: "some attestations overlap",
			inputs: []bitfield.Bitvector128{
				{0b00001001},
				{0b00010110},
				{0b00001010},
				{0b00110001},
			},
			want: []bitfield.Bitvector128{
				{0b00111011},
				{0b00011111},
			},
		},
		{
			name: "some attestations produce duplicates which are removed",
			inputs: []bitfield.Bitvector128{
				{0b00000101},
				{0b00000110},
				{0b00001010},
				{0b00001001},
			},
			want: []bitfield.Bitvector128{
				{0b00001111}, // both 0&1 and 2&3 produce this bitlist
			},
		},
		{
			name: "two attestations where one is fully contained within the other",
			inputs: []bitfield.Bitvector128{
				{0b00000001},
				{0b00000011},
			},
			want: []bitfield.Bitvector128{
				{0b00000011},
			},
		},
		{
			name: "two attestations where one is fully contained within the other reversed",
			inputs: []bitfield.Bitvector128{
				{0b00000011},
				{0b00000001},
			},
			want: []bitfield.Bitvector128{
				{0b00000011},
			},
		},
	}

	for _, tt := range tests {
		runner := func() {
			got, err := Aggregate(aggtesting.MakeSyncContributionsFromBitVector(tt.inputs))
			require.NoError(t, err)
			sort.Slice(got, func(i, j int) bool {
				return got[i].AggregationBits.Bytes()[0] < got[j].AggregationBits.Bytes()[0]
			})
			sort.Slice(tt.want, func(i, j int) bool {
				return tt.want[i].Bytes()[0] < tt.want[j].Bytes()[0]
			})
			assert.Equal(t, len(tt.want), len(got))
			for i, w := range tt.want {
				assert.DeepEqual(t, w.Bytes(), got[i].AggregationBits.Bytes())
			}
		}
		t.Run(fmt.Sprintf("%s/%s", tt.name, NaiveAggregation), func(t *testing.T) {
			runner()
		})
	}
}
