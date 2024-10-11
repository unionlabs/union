package validator

import (
	"bytes"
	"sort"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	v2 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestProposerSyncContributions_FilterByBlockRoot(t *testing.T) {
	rootA := [32]byte{'a'}
	rootB := [32]byte{'b'}
	var aggBits [fieldparams.SyncCommitteeAggregationBytesLength]byte
	tests := []struct {
		name string
		cs   proposerSyncContributions
		want proposerSyncContributions
	}{
		{
			name: "empty list",
			cs:   proposerSyncContributions{},
			want: proposerSyncContributions{},
		},
		{
			name: "single item, not found",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits[:]},
			},
			want: proposerSyncContributions{},
		},
		{
			name: "single item with filter, found",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], Slot: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], Slot: 1},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:]},
			},
		},
		{
			name: "multiple items with filter, found",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], Slot: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], Slot: 1},
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], Slot: 2},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], Slot: 3},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], Slot: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], Slot: 2},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cs := tt.cs.filterByBlockRoot(rootA)
			assert.DeepEqual(t, tt.want, cs)
		})
	}
}

func TestProposerSyncContributions_FilterBySubcommitteeID(t *testing.T) {
	rootA := [32]byte{'a'}
	rootB := [32]byte{'b'}
	var aggBits [fieldparams.SyncCommitteeAggregationBytesLength]byte
	tests := []struct {
		name string
		cs   proposerSyncContributions
		want proposerSyncContributions
	}{
		{
			name: "empty list",
			cs:   proposerSyncContributions{},
			want: proposerSyncContributions{},
		},
		{
			name: "single item, not found",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits[:], SubcommitteeIndex: 1},
			},
			want: proposerSyncContributions{},
		},
		{
			name: "single item with filter",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], SubcommitteeIndex: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], SubcommitteeIndex: 1},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:]},
			},
		},
		{
			name: "multiple items with filter",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], SubcommitteeIndex: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], SubcommitteeIndex: 1},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], SubcommitteeIndex: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], SubcommitteeIndex: 2},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{BlockRoot: rootA[:], SubcommitteeIndex: 0},
				&v2.SyncCommitteeContribution{BlockRoot: rootB[:], SubcommitteeIndex: 0},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cs := tt.cs.filterBySubIndex(0)
			assert.DeepEqual(t, tt.want, cs)
		})
	}
}

func TestProposerSyncContributions_Dedup(t *testing.T) {
	// Prepare aggregation bits for all scenarios
	var aggBits1, aggBits2_1, aggBits2_2, aggBits3, aggBits4_1, aggBits4_2, aggBits4_3, aggBits4_4, aggBits4_5, aggBits5_1, aggBits5_2, aggBits5_3, aggBits5_4, aggBits5_5, aggBits6, aggBits7_1, aggBits7_2, aggBits7_3, aggBits7_4, aggBits7_5, aggBits8_1, aggBits8_2, aggBits8_3, aggBits8_4, aggBits8_5, aggBits8_6, aggBits9_1, aggBits9_2, aggBits9_3, aggBits9_4, aggBits10_1, aggBits10_2, aggBits10_3, aggBits10_4, aggBits10_5, aggBits11_1, aggBits11_2, aggBits11_3, aggBits11_4, aggBits12_1, aggBits12_2, aggBits12_3, aggBits12_4, aggBits12_5, aggBits13_1, aggBits13_2 [fieldparams.SyncCommitteeAggregationBytesLength]byte
	b2_1, b2_2 := []byte{0b10111110, 0x01}, []byte{0b01111111, 0x01}
	copy(aggBits2_1[:], b2_1)
	copy(aggBits2_2[:], b2_2)
	b3 := []byte{0xba, 0x01}
	copy(aggBits3[:], b3)
	b4_1, b4_2, b4_3, b4_4, b4_5 := []byte{0b11001111, 0b1}, []byte{0b01101101, 0b1}, []byte{0b00101011, 0b1}, []byte{0b10100000, 0b1}, []byte{0b00010000, 0b1}
	copy(aggBits4_1[:], b4_1)
	copy(aggBits4_2[:], b4_2)
	copy(aggBits4_3[:], b4_3)
	copy(aggBits4_4[:], b4_4)
	copy(aggBits4_5[:], b4_5)
	b5_1, b5_2, b5_3, b5_4, b5_5 := []byte{0b11001111, 0b1}, []byte{0b01101101, 0b1}, []byte{0b00001111, 0b1}, []byte{0b00000011, 0b1}, []byte{0b00000001, 0b1}
	copy(aggBits5_1[:], b5_1)
	copy(aggBits5_2[:], b5_2)
	copy(aggBits5_3[:], b5_3)
	copy(aggBits5_4[:], b5_4)
	copy(aggBits5_5[:], b5_5)
	b6 := []byte{0b00000011, 0b1}
	copy(aggBits6[:], b6)
	b7_1, b7_2, b7_3, b7_4, b7_5 := []byte{0b01101101, 0b1}, []byte{0b00100010, 0b1}, []byte{0b10100101, 0b1}, []byte{0b00010000, 0b1}, []byte{0b11001111, 0b1}
	copy(aggBits7_1[:], b7_1)
	copy(aggBits7_2[:], b7_2)
	copy(aggBits7_3[:], b7_3)
	copy(aggBits7_4[:], b7_4)
	copy(aggBits7_5[:], b7_5)
	b8_1, b8_2, b8_3, b8_4, b8_5, b8_6 := []byte{0b00001111, 0b1}, []byte{0b11001111, 0b1}, []byte{0b10100101, 0b1}, []byte{0b00000001, 0b1}, []byte{0b00000011, 0b1}, []byte{0b01101101, 0b1}
	copy(aggBits8_1[:], b8_1)
	copy(aggBits8_2[:], b8_2)
	copy(aggBits8_3[:], b8_3)
	copy(aggBits8_4[:], b8_4)
	copy(aggBits8_5[:], b8_5)
	copy(aggBits8_6[:], b8_6)
	b9_1, b9_2, b9_3, b9_4 := []byte{0b00000101, 0b1}, []byte{0b00000011, 0b1}, []byte{0b10000001, 0b1}, []byte{0b00011001, 0b1}
	copy(aggBits9_1[:], b9_1)
	copy(aggBits9_2[:], b9_2)
	copy(aggBits9_3[:], b9_3)
	copy(aggBits9_4[:], b9_4)
	b10_1, b10_2, b10_3, b10_4, b10_5 := []byte{0b00001111, 0b1}, []byte{0b11001111, 0b1}, []byte{0b00000001, 0b1}, []byte{0b00000011, 0b1}, []byte{0b01101101, 0b1}
	copy(aggBits10_1[:], b10_1)
	copy(aggBits10_2[:], b10_2)
	copy(aggBits10_3[:], b10_3)
	copy(aggBits10_4[:], b10_4)
	copy(aggBits10_5[:], b10_5)
	b11_1, b11_2, b11_3, b11_4 := []byte{0b00000101, 0b1}, []byte{0b00000011, 0b1}, []byte{0b10000001, 0b1}, []byte{0b00011001, 0b1}
	copy(aggBits11_1[:], b11_1)
	copy(aggBits11_2[:], b11_2)
	copy(aggBits11_3[:], b11_3)
	copy(aggBits11_4[:], b11_4)
	b12_1, b12_2, b12_3, b12_4, b12_5 := []byte{0b00001111, 0b1}, []byte{0b11001111, 0b1}, []byte{0b00000001, 0b1}, []byte{0b00000011, 0b1}, []byte{0b01101101, 0b1}
	copy(aggBits12_1[:], b12_1)
	copy(aggBits12_2[:], b12_2)
	copy(aggBits12_3[:], b12_3)
	copy(aggBits12_4[:], b12_4)
	copy(aggBits12_5[:], b12_5)
	b13_1, b13_2 := []byte{0b00001111, 0b1}, []byte{0b11001111, 0b1}
	copy(aggBits13_1[:], b13_1)
	copy(aggBits13_2[:], b13_2)

	tests := []struct {
		name string
		cs   proposerSyncContributions
		want proposerSyncContributions
	}{
		{
			name: "nil list",
			cs:   nil,
			want: proposerSyncContributions(nil),
		},
		{
			name: "empty list",
			cs:   proposerSyncContributions{},
			want: proposerSyncContributions{},
		},
		{
			name: "single item",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits1[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits1[:]},
			},
		},
		{
			name: "two items no duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_2[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_1[:]},
			},
		},
		{
			name: "two items with duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits3[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits3[:]},
			},
		},
		{
			name: "sorted no duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_5[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_5[:]},
			},
		},
		{
			name: "sorted with duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_5[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits5_2[:]},
			},
		},
		{
			name: "all equal",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits6[:]},
			},
		},
		{
			name: "unsorted no duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_5[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_5[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits7_4[:]},
			},
		},
		{
			name: "unsorted with duplicates",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_5[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_4[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_6[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits8_3[:]},
			},
		},
		{
			name: "no proper subset (same root)",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_4[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits9_3[:]},
			},
		},
		{
			name: "proper subset (same root)",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_4[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_5[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits10_5[:]},
			},
		},
		{
			name: "no proper subset (different index)",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits11_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits11_2[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits11_3[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits11_4[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits11_4[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits11_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits11_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits11_1[:]},
			},
		},
		{
			name: "proper subset (different index 1)",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_1[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_2[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_1[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits12_3[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_4[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits12_3[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits12_5[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits12_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits12_5[:]},
			},
		},
		{
			name: "proper subset (different index 2)",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits13_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits13_2[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits13_1[:]},
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits13_2[:]},
			},
			want: proposerSyncContributions{
				&v2.SyncCommitteeContribution{SubcommitteeIndex: 1, AggregationBits: aggBits13_2[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits13_2[:]},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cs, err := tt.cs.dedup()
			if err != nil {
				t.Error(err)
			}
			sort.Slice(cs, func(i, j int) bool {
				if cs[i].AggregationBits.Count() == cs[j].AggregationBits.Count() {
					if cs[i].SubcommitteeIndex == cs[j].SubcommitteeIndex {
						return bytes.Compare(cs[i].AggregationBits, cs[j].AggregationBits) <= 0
					}
					return cs[i].SubcommitteeIndex > cs[j].SubcommitteeIndex
				}
				return cs[i].AggregationBits.Count() > cs[j].AggregationBits.Count()
			})
			assert.DeepEqual(t, tt.want, cs)
		})
	}
}

func TestProposerSyncContributions_MostProfitable(t *testing.T) {
	// Prepare aggregation bits for all scenarios.
	var aggBits1, aggBits2_1, aggBits2_2, aggBits3_1, aggBits3_2, aggBits4_1, aggBits4_2 [fieldparams.SyncCommitteeAggregationBytesLength]byte
	b1 := []byte{0b01}
	copy(aggBits1[:], b1)
	b2_1, b2_2 := []byte{0b01}, []byte{0b10}
	copy(aggBits2_1[:], b2_1)
	copy(aggBits2_2[:], b2_2)
	b3_1, b3_2 := []byte{0b0101}, []byte{0b0100}
	copy(aggBits3_1[:], b3_1)
	copy(aggBits3_2[:], b3_2)
	b4_1, b4_2 := []byte{0b0101}, []byte{0b0111}
	copy(aggBits4_1[:], b4_1)
	copy(aggBits4_2[:], b4_2)

	tests := []struct {
		name string
		cs   proposerSyncContributions
		want *v2.SyncCommitteeContribution
	}{
		{
			name: "Same item",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits1[:]},
			},
			want: &v2.SyncCommitteeContribution{AggregationBits: aggBits1[:]},
		},
		{
			name: "Same item again",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits2_2[:]},
			},
			want: &v2.SyncCommitteeContribution{AggregationBits: aggBits2_1[:]},
		},
		{
			name: "most profitable at the start",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits3_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits3_2[:]},
			},
			want: &v2.SyncCommitteeContribution{AggregationBits: aggBits3_1[:]},
		},
		{
			name: "most profitable at the end",
			cs: proposerSyncContributions{
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_1[:]},
				&v2.SyncCommitteeContribution{AggregationBits: aggBits4_2[:]},
			},
			want: &v2.SyncCommitteeContribution{AggregationBits: aggBits4_2[:]},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cs := tt.cs.mostProfitable()
			assert.DeepEqual(t, tt.want, cs)
		})
	}
}
