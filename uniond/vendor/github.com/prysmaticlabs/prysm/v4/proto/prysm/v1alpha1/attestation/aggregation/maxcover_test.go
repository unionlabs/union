package aggregation

import (
	"reflect"
	"sort"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	aggtesting "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestMaxCover_MaxCoverCandidates_filter(t *testing.T) {
	type args struct {
		covered       bitfield.Bitlist
		allowOverlaps bool
	}
	var problem MaxCoverCandidates
	tests := []struct {
		name string
		cl   MaxCoverCandidates
		args args
		want *MaxCoverCandidates
	}{
		{
			name: "nil list",
			cl:   nil,
			args: args{},
			want: &problem,
		},
		{
			name: "empty list",
			cl:   MaxCoverCandidates{},
			args: args{},
			want: &MaxCoverCandidates{},
		},
		{
			name: "all processed",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{2, &bitfield.Bitlist{0b01000010, 0b1}, 2, true},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{4, &bitfield.Bitlist{0b01000010, 0b1}, 2, true},
				{4, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
			},
			args: args{},
			want: &MaxCoverCandidates{},
		},
		{
			name: "partially processed",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{2, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{4, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
			},
			args: args{
				covered: bitfield.NewBitlist(8),
			},
			want: &MaxCoverCandidates{
				{2, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
			},
		},
		{
			name: "all overlapping",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b01000010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
			},
			args: args{
				covered: aggtesting.BitlistWithAllBitsSet(8),
			},
			want: &MaxCoverCandidates{},
		},
		{
			name: "partially overlapping",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b11000010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b01000011, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b10001010, 0b1}, 2, false},
			},
			args: args{
				covered: bitfield.Bitlist{0b10000001, 0b1},
			},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
			},
		},
		{
			name: "overlapping and processed and pending",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b11000010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{4, &bitfield.Bitlist{0b01000011, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b10001010, 0b1}, 2, false},
			},
			args: args{
				covered: bitfield.Bitlist{0b10000001, 0b1},
			},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
			},
		},
		{
			name: "overlapping and processed and pending - allow overlaps",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b11000010, 0b1}, 2, false},
				{3, &bitfield.Bitlist{0b00001010, 0b1}, 2, true},
				{4, &bitfield.Bitlist{0b01000011, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b10001010, 0b1}, 2, false},
			},
			args: args{
				covered:       bitfield.Bitlist{0b11111111, 0b1},
				allowOverlaps: true,
			},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00001010, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b11000010, 0b1}, 2, false},
				{4, &bitfield.Bitlist{0b10001010, 0b1}, 2, false},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.cl.filter(tt.args.covered, tt.args.allowOverlaps)
			if err != nil {
				t.Error(err)
			}
			sort.Slice(*got, func(i, j int) bool {
				return (*got)[i].key < (*got)[j].key
			})
			sort.Slice(*tt.want, func(i, j int) bool {
				return (*tt.want)[i].key < (*tt.want)[j].key
			})
			assert.DeepEqual(t, tt.want, got)
		})
	}
}

func TestMaxCover_MaxCoverCandidates_sort(t *testing.T) {
	var problem MaxCoverCandidates
	tests := []struct {
		name string
		cl   MaxCoverCandidates
		want *MaxCoverCandidates
	}{
		{
			name: "nil list",
			cl:   nil,
			want: &problem,
		},
		{
			name: "empty list",
			cl:   MaxCoverCandidates{},
			want: &MaxCoverCandidates{},
		},
		{
			name: "single item",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{}, 5, false},
			},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{}, 5, false},
			},
		},
		{
			name: "already sorted",
			cl: MaxCoverCandidates{
				{5, &bitfield.Bitlist{}, 5, false},
				{3, &bitfield.Bitlist{}, 4, false},
				{4, &bitfield.Bitlist{}, 4, false},
				{2, &bitfield.Bitlist{}, 2, false},
				{1, &bitfield.Bitlist{}, 1, false},
			},
			want: &MaxCoverCandidates{
				{5, &bitfield.Bitlist{}, 5, false},
				{3, &bitfield.Bitlist{}, 4, false},
				{4, &bitfield.Bitlist{}, 4, false},
				{2, &bitfield.Bitlist{}, 2, false},
				{1, &bitfield.Bitlist{}, 1, false},
			},
		},
		{
			name: "all equal",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
			},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
				{0, &bitfield.Bitlist{}, 5, false},
			},
		},
		{
			name: "unsorted",
			cl: MaxCoverCandidates{
				{2, &bitfield.Bitlist{}, 2, false},
				{4, &bitfield.Bitlist{}, 4, false},
				{3, &bitfield.Bitlist{}, 4, false},
				{5, &bitfield.Bitlist{}, 5, false},
				{1, &bitfield.Bitlist{}, 1, false},
			},
			want: &MaxCoverCandidates{
				{5, &bitfield.Bitlist{}, 5, false},
				{3, &bitfield.Bitlist{}, 4, false},
				{4, &bitfield.Bitlist{}, 4, false},
				{2, &bitfield.Bitlist{}, 2, false},
				{1, &bitfield.Bitlist{}, 1, false},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.cl.sort(); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("sort() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestMaxCover_MaxCoverCandidates_union(t *testing.T) {
	tests := []struct {
		name string
		cl   MaxCoverCandidates
		want bitfield.Bitlist
	}{
		{
			name: "nil",
			cl:   nil,
			want: bitfield.Bitlist(nil),
		},
		{
			name: "single empty candidate",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000000, 0b1}, 0, false},
			},
			want: bitfield.Bitlist{0b00000000, 0b1},
		},
		{
			name: "single full candidate",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b11111111, 0b1}, 8, false},
			},
			want: aggtesting.BitlistWithAllBitsSet(8),
		},
		{
			name: "mixed",
			cl: MaxCoverCandidates{
				{1, &bitfield.Bitlist{0b00000000, 0b00001110, 0b00001110, 0b1}, 6, false},
				{2, &bitfield.Bitlist{0b00000000, 0b01110000, 0b01110000, 0b1}, 6, false},
				{3, &bitfield.Bitlist{0b00000111, 0b10000001, 0b10000000, 0b1}, 6, false},
				{4, &bitfield.Bitlist{0b00000000, 0b00000110, 0b00000110, 0b1}, 4, false},
				{5, &bitfield.Bitlist{0b10000000, 0b00000001, 0b01100010, 0b1}, 4, false},
				{6, &bitfield.Bitlist{0b00001000, 0b00001000, 0b10000010, 0b1}, 4, false},
				{7, &bitfield.Bitlist{0b00000000, 0b00000001, 0b11111110, 0b1}, 8, false},
			},
			want: bitfield.Bitlist{0b10001111, 0b11111111, 0b11111110, 0b1},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got, err := tt.cl.union(); !reflect.DeepEqual(got, tt.want) || err != nil {
				t.Errorf("union(), got: %#b, %v, want: %#b", got, err, tt.want)
			}
		})
	}
}

func TestMaxCover_MaxCoverCandidates_score(t *testing.T) {
	var problem MaxCoverCandidates
	tests := []struct {
		name      string
		cl        MaxCoverCandidates
		uncovered bitfield.Bitlist
		want      *MaxCoverCandidates
	}{
		{
			name: "nil",
			cl:   nil,
			want: &problem,
		},
		{
			name: "uncovered set is empty",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 1, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 4, false},
				{2, &bitfield.Bitlist{0b00011011, 0b1}, 4, false},
				{3, &bitfield.Bitlist{0b00000001, 0b1}, 1, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 3, false},
			},
			uncovered: bitfield.NewBitlist(8),
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000001, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 0, false},
			},
		},
		{
			name: "completely uncovered",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000001, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 0, false},
			},
			uncovered: aggtesting.BitlistWithAllBitsSet(8),
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 1, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 4, false},
				{2, &bitfield.Bitlist{0b00011011, 0b1}, 4, false},
				{3, &bitfield.Bitlist{0b00000001, 0b1}, 1, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 3, false},
			},
		},
		{
			name: "partial uncovered set",
			cl: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 1, false},
				{2, &bitfield.Bitlist{0b10011011, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b11111111, 0b1}, 1, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 0, false},
			},
			uncovered: bitfield.Bitlist{0b11010010, 0b1},
			want: &MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000100, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00011011, 0b1}, 2, false},
				{2, &bitfield.Bitlist{0b10011011, 0b1}, 3, false},
				{3, &bitfield.Bitlist{0b11111111, 0b1}, 4, false},
				{4, &bitfield.Bitlist{0b00011010, 0b1}, 2, false},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got, err := tt.cl.score(tt.uncovered); !reflect.DeepEqual(got, tt.want) || err != nil {
				t.Errorf("score() = %v, %v, want %v", got, err, tt.want)
			}
		})
	}
}

func TestMaxCover_MaxCoverProblem_Cover(t *testing.T) {
	problemSet := func() MaxCoverCandidates {
		// test vectors originally from:
		// https://github.com/sigp/lighthouse/blob/master/beacon_node/operation_pool/src/max_cover.rs
		return MaxCoverCandidates{
			{0, &bitfield.Bitlist{0b00000100, 0b1}, 0, false},
			{1, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
			{2, &bitfield.Bitlist{0b00011011, 0b1}, 0, false},
			{3, &bitfield.Bitlist{0b00000001, 0b1}, 0, false},
			{4, &bitfield.Bitlist{0b00011010, 0b1}, 0, false},
		}
	}
	type args struct {
		k             int
		candidates    MaxCoverCandidates
		allowOverlaps bool
	}
	tests := []struct {
		name      string
		args      args
		want      *Aggregation
		wantedErr string
	}{
		{
			name:      "nil problem",
			args:      args{},
			wantedErr: ErrInvalidMaxCoverProblem.Error(),
		},
		{
			name: "different bitlengths",
			args: args{k: 3, candidates: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000000, 0b00011111, 0xf1}, 0, false},
				{2, &bitfield.Bitlist{0b00000001, 0b11100000, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000110, 0b00000000, 0b1}, 0, false},
			}},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b00000000, 0b00011111, 0xf1},
				Keys:     []int{0},
			},
		},
		{
			name: "k=0",
			args: args{k: 0, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0000000, 0b1},
				Keys:     []int{},
			},
		},
		{
			name: "k=1",
			args: args{k: 1, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0011011, 0b1},
				Keys:     []int{1},
			},
		},
		{
			name: "k=2",
			args: args{k: 2, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0011111, 0b1},
				Keys:     []int{1, 0},
			},
		},
		{
			name: "k=3",
			args: args{k: 3, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0011111, 0b1},
				Keys:     []int{1, 0},
			},
		},
		{
			name: "k=5",
			args: args{k: 5, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0011111, 0b1},
				Keys:     []int{1, 0},
			},
		},
		{
			name: "k=50",
			args: args{k: 50, candidates: problemSet()},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b0011111, 0b1},
				Keys:     []int{1, 0},
			},
		},
		{
			name: "suboptimal", // Greedy algorithm selects: 0, 2, 3, while 1,4,5 is optimal.
			args: args{k: 3, candidates: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000000, 0b00011111, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00000001, 0b11100000, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000110, 0b00000000, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00110000, 0b01110000, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b00000110, 0b10001100, 0b1}, 0, false},
				{5, &bitfield.Bitlist{0b01001001, 0b00000011, 0b1}, 0, false},
			}},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b00000111, 0b11111111, 0b1},
				Keys:     []int{0, 2, 3},
			},
		},
		{
			name: "allow overlaps",
			args: args{k: 5, allowOverlaps: true, candidates: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b00000000, 0b00000001, 0b11111110, 0b1}, 0, false},
				{1, &bitfield.Bitlist{0b00000000, 0b00001110, 0b00001110, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00000000, 0b01110000, 0b01110000, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000111, 0b10000001, 0b10000000, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b00000000, 0b00000110, 0b00000110, 0b1}, 0, false},
				{5, &bitfield.Bitlist{0b00000000, 0b00000001, 0b01100010, 0b1}, 0, false},
				{6, &bitfield.Bitlist{0b00001000, 0b00001000, 0b10000010, 0b1}, 0, false},
			}},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0b00001111, 0xff, 0b11111110, 0b1},
				Keys:     []int{0, 3, 1, 2, 6},
			},
		},
		{
			name: "empty bitlists",
			args: args{k: 5, allowOverlaps: true, candidates: MaxCoverCandidates{
				{1, &bitfield.Bitlist{0b0}, 0, false},
				{0, &bitfield.Bitlist{0b00000000, 0b00000000, 0b00000000, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00000000, 0b00000000, 0b00000000, 0b1}, 0, false},
			}},
			wantedErr: "empty bitlists: invalid max_cover problem",
		},
		{
			name: "overlapping solution dropped",
			args: args{k: 5, allowOverlaps: false, candidates: MaxCoverCandidates{
				{0, &bitfield.Bitlist{0b11111111, 0b11000111, 0b11111111, 0b1}, 0, false},
				// All remaining bitlists will overlap, so will be dropped.
				{1, &bitfield.Bitlist{0b11111111, 0b00001100, 0b11111111, 0b1}, 0, false},
				{2, &bitfield.Bitlist{0b00000000, 0b01110000, 0b01110000, 0b1}, 0, false},
				{3, &bitfield.Bitlist{0b00000111, 0b10000001, 0b10000000, 0b1}, 0, false},
				{4, &bitfield.Bitlist{0b00000000, 0b00000110, 0b00000110, 0b1}, 0, false},
				{5, &bitfield.Bitlist{0b00000000, 0b00000001, 0b01100010, 0b1}, 0, false},
				{6, &bitfield.Bitlist{0b00001000, 0b00001000, 0b10000010, 0b1}, 0, false},
			}},
			want: &Aggregation{
				Coverage: bitfield.Bitlist{0xff, 0b11000111, 0xff, 0b1},
				Keys:     []int{0},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mc := &MaxCoverProblem{
				Candidates: tt.args.candidates,
			}
			got, err := mc.Cover(tt.args.k, tt.args.allowOverlaps)
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
				assert.DeepEqual(t, tt.want, got)
			}
		})
	}
}

func TestMaxCover_MaxCover(t *testing.T) {
	problemSet := func() []*bitfield.Bitlist64 {
		return []*bitfield.Bitlist64{
			bitfield.NewBitlist64From([]uint64{0b00000100}),
			bitfield.NewBitlist64From([]uint64{0b00011011}),
			bitfield.NewBitlist64From([]uint64{0b00011011}),
			bitfield.NewBitlist64From([]uint64{0b00000001}),
			bitfield.NewBitlist64From([]uint64{0b00011010}),
		}
	}
	type args struct {
		k             int
		candidates    []*bitfield.Bitlist64
		allowOverlaps bool
	}
	type BitSetAggregation struct {
		Coverage *bitfield.Bitlist64
		Keys     []int
	}
	tests := []struct {
		name      string
		args      args
		want      *BitSetAggregation
		wantedErr string
	}{
		{
			name:      "nil problem",
			args:      args{},
			wantedErr: ErrInvalidMaxCoverProblem.Error(),
		},
		{
			name: "different bitlengths (pick first, combine with third)",
			args: args{k: 3, candidates: []*bitfield.Bitlist64{
				bitfield.NewBitlist64From([]uint64{0b00000001, 0b11100000, 0b10000000}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00011111}),
				bitfield.NewBitlist64From([]uint64{0b00000110, 0b00000000, 0b01000000}),
			}},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b00000111, 0b11100000, 0b11000000}),
				Keys:     []int{0, 2},
			},
		},
		{
			name: "different bitlengths (pick first, no other combination)",
			args: args{k: 3, candidates: []*bitfield.Bitlist64{
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00011111}),
				bitfield.NewBitlist64From([]uint64{0b00000001, 0b11100000, 0b1}),
				bitfield.NewBitlist64From([]uint64{0b00000110, 0b00000000, 0b1}),
			}},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b00000000, 0b00011111}),
				Keys:     []int{0},
			},
		},
		{
			name: "k=0",
			args: args{k: 0, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0}),
				Keys:     []int{},
			},
		},
		{
			name: "k=1",
			args: args{k: 1, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0011011}),
				Keys:     []int{1},
			},
		},
		{
			name: "k=2",
			args: args{k: 2, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0011111}),
				Keys:     []int{0, 1},
			},
		},
		{
			name: "k=3",
			args: args{k: 3, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0011111}),
				Keys:     []int{0, 1},
			},
		},
		{
			name: "k=5",
			args: args{k: 5, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0011111}),
				Keys:     []int{0, 1},
			},
		},
		{
			name: "k=50",
			args: args{k: 50, candidates: problemSet()},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b0011111}),
				Keys:     []int{0, 1},
			},
		},
		{
			name: "suboptimal", // Greedy algorithm selects: 0, 2, 3, while 1,4,5 is optimal.
			args: args{k: 3, candidates: []*bitfield.Bitlist64{
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00011111}),
				bitfield.NewBitlist64From([]uint64{0b00000001, 0b11100000}),
				bitfield.NewBitlist64From([]uint64{0b00000110, 0b00000000}),
				bitfield.NewBitlist64From([]uint64{0b00110000, 0b01110000}),
				bitfield.NewBitlist64From([]uint64{0b00000110, 0b10001100}),
				bitfield.NewBitlist64From([]uint64{0b01001001, 0b00000011}),
			}},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b00000111, 0b11111111}),
				Keys:     []int{0, 1, 2},
			},
		},
		{
			name: "allow overlaps",
			args: args{k: 5, allowOverlaps: true, candidates: []*bitfield.Bitlist64{
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00000001, 0b11111110}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00001110, 0b00001110}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b01110000, 0b01110000}),
				bitfield.NewBitlist64From([]uint64{0b00000111, 0b10000001, 0b10000000}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00000110, 0b00000110}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00000001, 0b01100010}),
				bitfield.NewBitlist64From([]uint64{0b00001000, 0b00001000, 0b10000010}),
			}},
			want: &BitSetAggregation{
				Coverage: bitfield.NewBitlist64From([]uint64{0b00001111, 0xff, 0b11111110}),
				Keys:     []int{0, 1, 2, 3, 6},
			},
		},
		{
			name: "empty bitlists",
			args: args{k: 5, allowOverlaps: true, candidates: []*bitfield.Bitlist64{
				bitfield.NewBitlist64From([]uint64{}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b00001110, 0b00001110}),
				bitfield.NewBitlist64From([]uint64{0b00000000, 0b01110000, 0b01110000}),
			}},
			wantedErr: "empty bitlists: invalid max_cover problem",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			selectedCandidates, coverage, err := MaxCover(tt.args.candidates, tt.args.k, tt.args.allowOverlaps)
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
				assert.DeepEqual(t, tt.want.Coverage, coverage)
				selectedKeys := make([]int, selectedCandidates.Count())
				selectedCandidates.NoAllocBitIndices(selectedKeys)
				assert.DeepEqual(t, tt.want.Keys, selectedKeys)
			}
		})
	}
}
