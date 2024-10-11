package attestations

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestAggregateAttestations_MaxCover_NewMaxCover(t *testing.T) {
	type args struct {
		atts []*ethpb.Attestation
	}
	tests := []struct {
		name string
		args args
		want *aggregation.MaxCoverProblem
	}{
		{
			name: "nil attestations",
			args: args{
				atts: nil,
			},
			want: &aggregation.MaxCoverProblem{Candidates: []*aggregation.MaxCoverCandidate{}},
		},
		{
			name: "no attestations",
			args: args{
				atts: []*ethpb.Attestation{},
			},
			want: &aggregation.MaxCoverProblem{Candidates: []*aggregation.MaxCoverCandidate{}},
		},
		{
			name: "single attestation",
			args: args{
				atts: []*ethpb.Attestation{
					{AggregationBits: bitfield.Bitlist{0b00001010, 0b1}},
				},
			},
			want: &aggregation.MaxCoverProblem{
				Candidates: aggregation.MaxCoverCandidates{
					aggregation.NewMaxCoverCandidate(0, &bitfield.Bitlist{0b00001010, 0b1}),
				},
			},
		},
		{
			name: "multiple attestations",
			args: args{
				atts: []*ethpb.Attestation{
					{AggregationBits: bitfield.Bitlist{0b00001010, 0b1}},
					{AggregationBits: bitfield.Bitlist{0b00101010, 0b1}},
					{AggregationBits: bitfield.Bitlist{0b11111010, 0b1}},
					{AggregationBits: bitfield.Bitlist{0b00000010, 0b1}},
					{AggregationBits: bitfield.Bitlist{0b00000001, 0b1}},
				},
			},
			want: &aggregation.MaxCoverProblem{
				Candidates: aggregation.MaxCoverCandidates{
					aggregation.NewMaxCoverCandidate(0, &bitfield.Bitlist{0b00001010, 0b1}),
					aggregation.NewMaxCoverCandidate(1, &bitfield.Bitlist{0b00101010, 0b1}),
					aggregation.NewMaxCoverCandidate(2, &bitfield.Bitlist{0b11111010, 0b1}),
					aggregation.NewMaxCoverCandidate(3, &bitfield.Bitlist{0b00000010, 0b1}),
					aggregation.NewMaxCoverCandidate(4, &bitfield.Bitlist{0b00000001, 0b1}),
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.DeepEqual(t, tt.want, NewMaxCover(tt.args.atts))
		})
	}
}

func TestAggregateAttestations_MaxCover_AttList_validate(t *testing.T) {
	tests := []struct {
		name      string
		atts      attList
		wantedErr string
	}{
		{
			name:      "nil list",
			atts:      nil,
			wantedErr: "nil list",
		},
		{
			name:      "empty list",
			atts:      attList{},
			wantedErr: "empty list",
		},
		{
			name:      "first bitlist is nil",
			atts:      attList{&ethpb.Attestation{}},
			wantedErr: "bitlist cannot be nil or empty",
		},
		{
			name: "non first bitlist is nil",
			atts: attList{
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
				&ethpb.Attestation{},
			},
			wantedErr: "bitlist cannot be nil or empty",
		},
		{
			name: "first bitlist is empty",
			atts: attList{
				&ethpb.Attestation{AggregationBits: bitfield.Bitlist{}},
			},
			wantedErr: "bitlist cannot be nil or empty",
		},
		{
			name: "non first bitlist is empty",
			atts: attList{
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
				&ethpb.Attestation{AggregationBits: bitfield.Bitlist{}},
			},
			wantedErr: "bitlist cannot be nil or empty",
		},
		{
			name: "valid bitlists",
			atts: attList{
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
				&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(64)},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.atts.validate()
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestAggregateAttestations_rearrangeProcessedAttestations(t *testing.T) {
	tests := []struct {
		name     string
		atts     []*ethpb.Attestation
		keys     []int
		wantAtts []*ethpb.Attestation
	}{
		{
			name: "nil attestations",
		},
		{
			name: "single attestation no processed keys",
			atts: []*ethpb.Attestation{
				{},
			},
			wantAtts: []*ethpb.Attestation{
				{},
			},
		},
		{
			name: "single attestation processed",
			atts: []*ethpb.Attestation{
				{},
			},
			keys: []int{0},
			wantAtts: []*ethpb.Attestation{
				nil,
			},
		},
		{
			name: "multiple processed, last attestation marked",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
			},
			keys: []int{1, 4}, // Only attestation at index 1, should be moved, att at 4 is already at the end.
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				nil, nil,
			},
		},
		{
			name: "all processed",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
			},
			keys: []int{0, 1, 2, 3, 4},
			wantAtts: []*ethpb.Attestation{
				nil, nil, nil, nil, nil,
			},
		},
		{
			name: "operate on slice, single attestation marked",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				// Assuming some attestations have been already marked as nil, during previous rounds:
				nil, nil, nil,
			},
			keys: []int{2},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				nil, nil, nil, nil,
			},
		},
		{
			name: "operate on slice, non-last attestation marked",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				{AggregationBits: bitfield.Bitlist{0x05}},
				// Assuming some attestations have been already marked as nil, during previous rounds:
				nil, nil, nil,
			},
			keys: []int{2, 3},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x05}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				nil, nil, nil, nil, nil,
			},
		},
		{
			name: "operate on slice, last attestation marked",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				// Assuming some attestations have been already marked as nil, during previous rounds:
				nil, nil, nil,
			},
			keys: []int{2, 4},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				nil, nil, nil, nil, nil,
			},
		},
		{
			name: "many items, many selected, keys unsorted",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
				{AggregationBits: bitfield.Bitlist{0x02}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				{AggregationBits: bitfield.Bitlist{0x04}},
				{AggregationBits: bitfield.Bitlist{0x05}},
				{AggregationBits: bitfield.Bitlist{0x06}},
			},
			keys: []int{4, 1, 2, 5, 6},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x03}},
				nil, nil, nil, nil, nil,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			candidates := make([]*bitfield.Bitlist64, len(tt.atts))
			for i := 0; i < len(tt.atts); i++ {
				if tt.atts[i] != nil {
					var err error
					candidates[i], err = tt.atts[i].AggregationBits.ToBitlist64()
					if err != nil {
						t.Error(err)
					}
				}
			}
			rearrangeProcessedAttestations(tt.atts, candidates, tt.keys)
			assert.DeepEqual(t, tt.atts, tt.wantAtts)
		})
	}
}

func TestAggregateAttestations_aggregateAttestations(t *testing.T) {
	sign := bls.NewAggregateSignature().Marshal()
	tests := []struct {
		name          string
		atts          []*ethpb.Attestation
		wantAtts      []*ethpb.Attestation
		keys          []int
		coverage      *bitfield.Bitlist64
		wantTargetIdx int
		wantErr       string
	}{
		{
			name:          "nil attestation",
			wantTargetIdx: 0,
			wantErr:       ErrInvalidAttestationCount.Error(),
			keys:          []int{0, 1, 2},
		},
		{
			name: "single attestation",
			atts: []*ethpb.Attestation{
				{},
			},
			wantTargetIdx: 0,
			wantErr:       ErrInvalidAttestationCount.Error(),
			keys:          []int{0, 1, 2},
		},
		{
			name:          "no keys",
			wantTargetIdx: 0,
			wantErr:       ErrInvalidAttestationCount.Error(),
		},
		{
			name: "two attestations, none selected",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
			},
			wantTargetIdx: 0,
			wantErr:       ErrInvalidAttestationCount.Error(),
			keys:          []int{},
		},
		{
			name: "two attestations, one selected",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x00}},
				{AggregationBits: bitfield.Bitlist{0x01}},
			},
			wantTargetIdx: 0,
			wantErr:       ErrInvalidAttestationCount.Error(),
			keys:          []int{0},
		},
		{
			name: "two attestations, both selected, empty coverage",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0b00000001, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0b00000110, 0b1}, Signature: sign},
			},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0b00000111, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0b00000110, 0b1}, Signature: sign},
			},
			wantTargetIdx: 0,
			wantErr:       "invalid or empty coverage",
			keys:          []int{0, 1},
		},
		{
			name: "two attestations, both selected",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000001, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000010, 0b1}, Signature: sign},
			},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000011, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000010, 0b1}, Signature: sign},
			},
			wantTargetIdx: 0,
			keys:          []int{0, 1},
			coverage: func() *bitfield.Bitlist64 {
				b, err := bitfield.NewBitlist64FromBytes(64, []byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000011})
				if err != nil {
					t.Fatal(err)
				}
				return b
			}(),
		},
		{
			name: "many attestations, several selected",
			atts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000001, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000010, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000100, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00001000, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00010000, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00100000, 0b1}, Signature: sign},
			},
			wantAtts: []*ethpb.Attestation{
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000001, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00010110, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00000100, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00001000, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00010000, 0b1}, Signature: sign},
				{AggregationBits: bitfield.Bitlist{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00100000, 0b1}, Signature: sign},
			},
			wantTargetIdx: 1,
			keys:          []int{1, 2, 4},
			coverage: func() *bitfield.Bitlist64 {
				b, err := bitfield.NewBitlist64FromBytes(64, []byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0b00010110})
				if err != nil {
					t.Fatal(err)
				}
				return b
			}(),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotTargetIdx, err := aggregateAttestations(tt.atts, tt.keys, tt.coverage)
			if tt.wantErr != "" {
				assert.ErrorContains(t, tt.wantErr, err)
				return
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantTargetIdx, gotTargetIdx)
			extractBitlists := func(atts []*ethpb.Attestation) []bitfield.Bitlist {
				bl := make([]bitfield.Bitlist, len(atts))
				for i, att := range atts {
					bl[i] = att.AggregationBits
				}
				return bl
			}
			assert.DeepEqual(t, extractBitlists(tt.atts), extractBitlists(tt.wantAtts))
		})
	}
}
