package version

import "testing"

func TestVersionString(t *testing.T) {
	tests := []struct {
		name    string
		version int
		want    string
	}{
		{
			name:    "phase0",
			version: Phase0,
			want:    "phase0",
		},
		{
			name:    "altair",
			version: Altair,
			want:    "altair",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := String(tt.version); got != tt.want {
				t.Errorf("String() = %v, want %v", got, tt.want)
			}
		})
	}
}
