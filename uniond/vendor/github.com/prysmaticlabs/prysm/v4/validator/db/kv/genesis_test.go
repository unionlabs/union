package kv

import (
	"context"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_GenesisValidatorsRoot_ReadAndWrite(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name    string
		init    []byte
		want    []byte
		write   []byte
		wantErr bool
	}{
		{
			name:    "empty then write",
			init:    nil,
			want:    params.BeaconConfig().ZeroHash[:],
			write:   params.BeaconConfig().ZeroHash[:],
			wantErr: false,
		},
		{
			name:    "zero then overwrite with the same value",
			init:    params.BeaconConfig().ZeroHash[:],
			want:    params.BeaconConfig().ZeroHash[:],
			write:   params.BeaconConfig().ZeroHash[:],
			wantErr: false,
		},
		{
			name:    "zero then overwrite with a different value",
			init:    params.BeaconConfig().ZeroHash[:],
			want:    params.BeaconConfig().ZeroHash[:],
			write:   []byte{5},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Initialize the database with the initial value.
			db := setupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
			err := db.SaveGenesisValidatorsRoot(ctx, tt.init)
			require.NoError(t, err)

			// Read the value from the database (just to ensure our setup is OK).
			got, err := db.GenesisValidatorsRoot(ctx)
			require.NoError(t, err)
			require.DeepEqual(t, tt.init, got)

			// Write the value to the database.
			err = db.SaveGenesisValidatorsRoot(ctx, tt.write)
			if (err != nil) != tt.wantErr {
				t.Errorf("GenesisValidatorsRoot() error = %v, wantErr %v", err, tt.wantErr)
			}

			// Read the value from the database.
			got, err = db.GenesisValidatorsRoot(ctx)
			require.NoError(t, err)
			require.DeepEqual(t, tt.want, got)

			// Close the database.
			require.NoError(t, db.Close())
		})
	}
}
