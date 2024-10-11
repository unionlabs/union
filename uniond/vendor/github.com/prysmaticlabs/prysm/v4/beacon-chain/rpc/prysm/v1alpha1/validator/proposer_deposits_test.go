package validator

import "testing"

func TestShouldFallback(t *testing.T) {
	tests := []struct {
		name            string
		totalDepCount   uint64
		unFinalizedDeps uint64
		want            bool
	}{
		{
			name:            "0 dep count",
			totalDepCount:   0,
			unFinalizedDeps: 100,
			want:            false,
		},
		{
			name:            "0 unfinalized count",
			totalDepCount:   100,
			unFinalizedDeps: 0,
			want:            false,
		},
		{
			name:            "equal number of deposits and non finalized deposits",
			totalDepCount:   1000,
			unFinalizedDeps: 1000,
			want:            true,
		},
		{
			name:            "large number of non finalized deposits",
			totalDepCount:   300000,
			unFinalizedDeps: 100000,
			want:            true,
		},
		{
			name:            "small number of non finalized deposits",
			totalDepCount:   300000,
			unFinalizedDeps: 2000,
			want:            false,
		},
		{
			name:            "unfinalized deposits beyond threshold",
			totalDepCount:   300000,
			unFinalizedDeps: 10000,
			want:            true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := shouldRebuildTrie(tt.totalDepCount, tt.unFinalizedDeps); got != tt.want {
				t.Errorf("shouldRebuildTrie() = %v, want %v", got, tt.want)
			}
		})
	}
}
