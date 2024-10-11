package keeper

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestValidateAddress(t *testing.T) {
	specs := map[string]struct {
		human string
		valid bool
	}{
		"valid address": {
			human: RandomBech32AccountAddress(t),
			valid: true,
		},
		"invalid address": {
			human: "cosmos1invalid",
			valid: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			_, err := validateAddress(spec.human)

			if spec.valid {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
			}
		})
	}
}
