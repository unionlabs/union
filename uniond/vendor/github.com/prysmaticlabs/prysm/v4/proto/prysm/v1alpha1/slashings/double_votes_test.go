package slashings

import (
	"testing"
)

func TestSigningRootsDiffer(t *testing.T) {
	type args struct {
		existingSigningRoot []byte
		incomingSigningRoot []byte
	}
	tests := []struct {
		name string
		args args
		want bool
	}{
		{
			name: "nil existing signing root is slashable",
			args: args{
				existingSigningRoot: nil,
				incomingSigningRoot: []byte{1},
			},
			want: true,
		},
		{
			name: "empty existing signing root is slashable",
			args: args{
				existingSigningRoot: []byte{},
				incomingSigningRoot: []byte{1},
			},
			want: true,
		},
		{
			name: "Non-empty, different existing signing root is slashable",
			args: args{
				existingSigningRoot: []byte{2},
				incomingSigningRoot: []byte{1},
			},
			want: true,
		},
		{
			name: "Non-empty, same existing signing root and incoming signing root is not slashable",
			args: args{
				existingSigningRoot: []byte{2},
				incomingSigningRoot: []byte{2},
			},
			want: false,
		},
		{
			name: "Both nil are considered slashable",
			args: args{
				existingSigningRoot: nil,
				incomingSigningRoot: nil,
			},
			want: true,
		},
		{
			name: "Both empty are considered slashable",
			args: args{
				existingSigningRoot: []byte{},
				incomingSigningRoot: []byte{},
			},
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := SigningRootsDiffer(tt.args.existingSigningRoot, tt.args.incomingSigningRoot); got != tt.want {
				t.Errorf("SigningRootsDiffer() = %v, want %v", got, tt.want)
			}
		})
	}
}
