package slashings

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func TestIsSurround(t *testing.T) {
	type args struct {
		a *ethpb.IndexedAttestation
		b *ethpb.IndexedAttestation
	}
	tests := []struct {
		name string
		args args
		want bool
	}{
		{
			name: "0 values returns false",
			args: args{
				a: createAttestation(0, 0),
				b: createAttestation(0, 0),
			},
			want: false,
		},
		{
			name: "detects surrounding attestation",
			args: args{
				a: createAttestation(2, 5),
				b: createAttestation(3, 4),
			},
			want: true,
		},
		{
			name: "new attestation source == old source, but new target < old target",
			args: args{
				a: createAttestation(3, 5),
				b: createAttestation(3, 4),
			},
			want: false,
		},
		{
			name: "new attestation source > old source, but new target == old target",
			args: args{
				a: createAttestation(3, 5),
				b: createAttestation(4, 5),
			},
			want: false,
		},
		{
			name: "new attestation source and targets equal to old one",
			args: args{
				a: createAttestation(3, 5),
				b: createAttestation(3, 5),
			},
			want: false,
		},
		{
			name: "new attestation source == old source, but new target > old target",
			args: args{
				a: createAttestation(3, 5),
				b: createAttestation(3, 6),
			},
			want: false,
		},
		{
			name: "new attestation source < old source, but new target == old target",
			args: args{
				a: createAttestation(3, 5),
				b: createAttestation(2, 5),
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := IsSurround(tt.args.a, tt.args.b); got != tt.want {
				t.Errorf("IsSurrounding() = %v, want %v", got, tt.want)
			}
		})
	}
}

func createAttestation(source, target primitives.Epoch) *ethpb.IndexedAttestation {
	return &ethpb.IndexedAttestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{Epoch: source},
			Target: &ethpb.Checkpoint{Epoch: target},
		},
	}
}
