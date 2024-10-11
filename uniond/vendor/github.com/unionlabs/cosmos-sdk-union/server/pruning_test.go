package server

import (
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/require"

	pruningtypes "cosmossdk.io/store/pruning/types"
)

func TestGetPruningOptionsFromFlags(t *testing.T) {
	tests := []struct {
		name            string
		initParams      func() *viper.Viper
		expectedOptions pruningtypes.PruningOptions
		wantErr         bool
	}{
		{
			name: FlagPruning,
			initParams: func() *viper.Viper {
				v := viper.New()
				v.Set(FlagPruning, pruningtypes.PruningOptionNothing)
				return v
			},
			expectedOptions: pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
		},
		{
			name: "custom pruning options",
			initParams: func() *viper.Viper {
				v := viper.New()
				v.Set(FlagPruning, pruningtypes.PruningOptionCustom)
				v.Set(FlagPruningKeepRecent, 1234)
				v.Set(FlagPruningInterval, 10)

				return v
			},
			expectedOptions: pruningtypes.NewCustomPruningOptions(1234, 10),
		},
		{
			name: pruningtypes.PruningOptionDefault,
			initParams: func() *viper.Viper {
				v := viper.New()
				v.Set(FlagPruning, pruningtypes.PruningOptionDefault)
				return v
			},
			expectedOptions: pruningtypes.NewPruningOptions(pruningtypes.PruningDefault),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(j *testing.T) {
			viper.Reset()
			viper.SetDefault(FlagPruning, pruningtypes.PruningOptionDefault)
			v := tt.initParams()

			opts, err := GetPruningOptionsFromFlags(v)
			if tt.wantErr {
				require.Error(t, err)
				return
			}

			require.Equal(t, tt.expectedOptions, opts)
		})
	}
}
